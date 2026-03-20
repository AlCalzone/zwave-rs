use super::{AwaitedCC, DriverActor, DriverInput};
use crate::error::{Error, Result};
use futures::{channel::oneshot, select_biased, FutureExt, StreamExt};
use std::sync::Arc;
use std::time::Instant;
use zwave_cc::commandclass::{CCSession, CcOrRaw};
use zwave_cc::prelude::*;
use zwave_core::prelude::*;
use zwave_core::security::{
    SecurityManager, SecurityManager2, SecurityManager2Storage, SecurityManagerOptions,
    SecurityManagerStorage,
};
use zwave_core::{log::Loglevel, util::MaybeSleep};
use zwave_logging::loggers::node::NodeLogger;
use zwave_logging::{
    loggers::{controller::ControllerLogger, driver::DriverLogger},
    Direction, LocalImmutableLogger, LogInfo,
};
use zwave_serial::prelude::*;

impl DriverActor {
    pub async fn run(&mut self) {
        loop {
            // Figure out if there is a timeout we need to wait for
            let min_sleep_duration = self
                .awaited_ccs
                .iter()
                .filter_map(|cc| cc.timeout)
                .min()
                .map(|t| t - Instant::now());
            let maybe_sleep = MaybeSleep::new(min_sleep_duration);

            select_biased! {
                // Handle inputs
                input = self.input_rx.next() => {
                    if let Some(input) = input {
                        self.handle_input(input);
                    }
                },
                // before timeouts
                _ = maybe_sleep.fuse() => {
                    self.handle_timeouts();
                }
            }
        }
    }

    pub fn driver_log(&self) -> DriverLogger<'_> {
        DriverLogger::new(self)
    }

    pub fn controller_log(&self) -> ControllerLogger<'_> {
        ControllerLogger::new(self)
    }

    pub(crate) fn node_log(&self, node_id: NodeId, endpoint: EndpointIndex) -> NodeLogger<'_> {
        NodeLogger::new(self, node_id, endpoint)
    }

    /// Passes an input that the driver needs to handle
    fn handle_input(&mut self, input: DriverInput) {
        match input {
            DriverInput::Unsolicited { command } => {
                self.handle_unsolicited_command(command);
            }

            DriverInput::Log { log, level } => {
                self.log_queue
                    .try_send((log, level))
                    .expect("Failed to log message");
            }

            DriverInput::AwaitCC {
                predicate,
                timeout,
                callback,
            } => {
                let awaited_cc = AwaitedCC {
                    predicate,
                    timeout: timeout.map(|t| Instant::now() + t),
                    callback,
                };
                self.awaited_ccs.push(awaited_cc);
            }
            DriverInput::InitSecurityManagers => {
                self.init_security_managers();
            }
        }
    }

    fn handle_timeouts(&mut self) {
        // Figure out which timeout(s) elapsed and take them out of the awaited list
        let now = Instant::now();
        let mut awaited_ccs = Vec::new();
        for cc in self.awaited_ccs.drain(..) {
            // Preserve the awaited CCs that haven't timed out yet
            if cc.timeout.map(|t| now >= t).unwrap_or(false) {
                awaited_ccs.push(cc);
            } else {
                // This CC has timed out, send an error to the callback
                let _ = cc.callback.send(Err(Error::Timeout));
            }
        }
        self.awaited_ccs = awaited_ccs;
    }

    fn take_matching_awaited_cc(
        &mut self,
        cc: &WithAddress<CC>,
    ) -> Option<oneshot::Sender<Result<WithAddress<CC>>>> {
        let index = self.awaited_ccs.iter().position(|a| (a.predicate)(cc));
        index.map(|i| self.awaited_ccs.remove(i).callback)
    }

    fn get_cc_parsing_context(&self, address: &CCAddress) -> CCParsingContext {
        CCParsingContext::builder()
            .home_id(self.serial_api.storage.home_id().get())
            .source_node_id(address.source_node_id)
            .frame_addressing(Some((&address.destination).into()))
            .own_node_id(self.serial_api.storage.own_node_id().get())
            .security_manager(self.storage.security_manager().cloned())
            .security_manager2(self.storage.security_manager2().cloned())
            .build()
    }

    fn handle_unsolicited_command(&mut self, mut command: Command) {
        // Figure out if this is a command that contains a CC...
        let cc = match &mut command {
            Command::ApplicationCommandRequest(cmd) => Some(&mut cmd.command),
            Command::BridgeApplicationCommandRequest(cmd) => Some(&mut cmd.command),
            _ => None,
        };
        // ...and handle it
        if let Some(cc) = cc {
            // FIXME: Can we get rid of all these clones()?
            let (address, cc_or_raw) = cc.as_parts_mut();

            let ctx = self.get_cc_parsing_context(address);
            match cc_or_raw.clone().try_as_cc(ctx) {
                Ok(parsed_cc) => {
                    // Update the command, so it gets logged correctly
                    *cc_or_raw = CcOrRaw::CC(parsed_cc);
                }
                Err(e) => {
                    self.driver_log()
                        .error(|| format!("failed to parse CC: {}", e));
                    return;
                }
            };

            // TODO: This back and forth is pretty awkward
            let CcOrRaw::CC(cc) = cc_or_raw else {
                panic!("The CC should have been parsed already")
            };
            let mut cc = cc.clone().with_address(address.clone());

            // Check if there is someone waiting for this CC
            if let Some(callback) = self.take_matching_awaited_cc(&cc) {
                self.node_log(cc.address().source_node_id, cc.address().endpoint_index)
                    .command(&command, Direction::Inbound);

                let _ = callback.send(Ok(cc));
                return;
            }

            let node_logger =
                self.node_log(cc.address().source_node_id, cc.address().endpoint_index);

            // Check if the CC is split across multiple partial CCs
            if let Some(session_id) = cc.session_id() {
                // FIXME: Look up other partial CCs and pass them to merge_session
                // If so, try to merge it
                let ctx = self.get_cc_parsing_context(cc.address());
                if let Err(e) = cc.merge_session(ctx, vec![]) {
                    node_logger.error(|| format!("failed to merge partial CCs: {}", e));
                    return;
                }
            }

            node_logger.command(&command, Direction::Inbound);
        } else {
            self.controller_log().command(&command, Direction::Inbound);
        }
    }

    fn init_security_managers(&mut self) {
        let logger = self.driver_log();

        if let Some(ref s0_key) = self.security_keys.s0_legacy {
            logger.info(|| "Network key for S0 configured, enabling S0 security manager...");
            let storage = SecurityManagerStorage::new(SecurityManagerOptions {
                own_node_id: self.serial_api.storage.own_node_id().get(),
                network_key: s0_key.clone(),
            });
            let sec_man = SecurityManager::new(Arc::new(storage));
            let _ = self.storage.security_manager().replace(Some(sec_man));
        } else {
            logger.warn(|| "No network key for S0 configured, communication with secure (S0) devices won't work!");
        }

        let has_s2_keys = self.security_keys.s2_unauthenticated.is_some()
            || self.security_keys.s2_authenticated.is_some()
            || self.security_keys.s2_access_control.is_some();

        if has_s2_keys {
            logger.info(|| "S2 network keys configured, enabling S2 security manager...");
            let sec_man = SecurityManager2::new(Arc::new(SecurityManager2Storage::new()));

            if let Some(ref key) = self.security_keys.s0_legacy {
                sec_man.set_key(SecurityClass::S0Legacy, key.clone());
            }
            if let Some(ref key) = self.security_keys.s2_unauthenticated {
                sec_man.set_key(SecurityClass::S2Unauthenticated, key.clone());
            }
            if let Some(ref key) = self.security_keys.s2_authenticated {
                sec_man.set_key(SecurityClass::S2Authenticated, key.clone());
            }
            if let Some(ref key) = self.security_keys.s2_access_control {
                sec_man.set_key(SecurityClass::S2AccessControl, key.clone());
            }

            let _ = self.storage.security_manager2().replace(Some(sec_man));
        } else {
            logger.warn(|| "No network keys for S2 configured, communication with secure (S2) devices won't work!");
        }
    }
}

impl LocalImmutableLogger for DriverActor {
    fn log(&self, log: LogInfo, level: Loglevel) {
        let _ = self.log_queue.clone().try_send((log, level));
    }

    fn log_level(&self) -> Loglevel {
        Loglevel::Debug
    }

    fn set_log_level(&self, level: Loglevel) {
        todo!()
    }
}
