use super::{
    awaited::{AwaitedRef, AwaitedRegistry, Predicate},
    storage::DriverStorage,
    DriverOptionsStatic, SerialListener, TaskCommandReceiver, TaskCommandSender,
};
use crate::{
    cache::ValueCache, define_async_task_commands, driver_api::DriverApi, BackgroundTask,
    DriverOptions,
};
use std::{ops::Deref, sync::Arc, time::Duration};
use tokio::sync::Notify;
use zwave_cc::commandclass::{
    CCAddress, CCAddressable, CCParsingContext, CCSession, CCValues, SecurityCCNonceGet,
    SecurityCCNonceReport, WithAddress, CC,
};
use zwave_core::{
    cache::Cache,
    prelude::*,
    security::{SecurityManager, SecurityManagerOptions},
    util::now,
    value_id::EndpointValueId,
    wrapping_counter::WrappingCounter,
};
use zwave_logging::Direction;
use zwave_serial::{
    command::{Command, CommandParsingContext},
    frame::{ControlFlow, SerialFrame},
};

define_async_task_commands!(MainTaskCommand {
    RegisterAwaitedCC -> AwaitedRef<WithAddress<CC>> {
        predicate: Predicate<WithAddress<CC>>,
        timeout: Option<Duration>
    },
    RegisterAwaitedCommand -> AwaitedRef<Command> {
        predicate: Predicate<Command>,
        timeout: Option<Duration>
    },
    RegisterAwaitedControlFlowFrame -> AwaitedRef<ControlFlow> {
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>
    },
    GetNextCallbackId -> u8 {},
});

pub(crate) type MainTaskCommandSender = TaskCommandSender<MainTaskCommand>;
pub(crate) type MainTaskCommandReceiver = TaskCommandReceiver<MainTaskCommand>;

pub(crate) struct MainLoop {
    driver_api: DriverApi,
    driver_options: DriverOptionsStatic,

    shutdown: Arc<Notify>,
    cmd_rx: MainTaskCommandReceiver,
    serial_listener: SerialListener,

    awaited_control_flow_frames: Arc<AwaitedRegistry<ControlFlow>>,
    awaited_commands: Arc<AwaitedRegistry<Command>>,
    awaited_ccs: Arc<AwaitedRegistry<WithAddress<CC>>>,
    callback_id_gen: WrappingCounter<u8>,
}

impl MainLoop {
    pub fn new(
        driver_api: DriverApi,
        driver_options: DriverOptionsStatic,
        shutdown: Arc<Notify>,
        cmd_rx: MainTaskCommandReceiver,
        serial_listener: SerialListener,
    ) -> Self {
        Self {
            driver_api,
            driver_options,

            shutdown,
            cmd_rx,
            serial_listener,

            awaited_control_flow_frames: Arc::new(AwaitedRegistry::default()),
            awaited_commands: Arc::new(AwaitedRegistry::default()),
            awaited_ccs: Arc::new(AwaitedRegistry::default()),
            callback_id_gen: WrappingCounter::new(),
        }
    }

    async fn handle_command(&mut self, cmd: MainTaskCommand) {
        match cmd {
            MainTaskCommand::RegisterAwaitedControlFlowFrame(ctrl) => {
                let result = self
                    .awaited_control_flow_frames
                    .add(ctrl.predicate, ctrl.timeout);
                ctrl.callback
                    .send(result)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::RegisterAwaitedCommand(cmd) => {
                let result = self.awaited_commands.add(cmd.predicate, cmd.timeout);
                cmd.callback
                    .send(result)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::RegisterAwaitedCC(cc) => {
                let result = self.awaited_ccs.add(cc.predicate, cc.timeout);
                cc.callback
                    .send(result)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::GetNextCallbackId(cmd) => {
                let id = self.callback_id_gen.increment();
                cmd.callback
                    .send(id)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }
        }
    }

    fn get_command_parsing_context(&mut self) -> CommandParsingContext {
        CommandParsingContext::builder()
            .own_node_id(self.driver_api.own_node_id())
            .node_id_type(self.driver_api.storage.node_id_type())
            .sdk_version(self.driver_api.storage.sdk_version())
            .security_manager(self.driver_api.security_manager())
            .build()
    }

    fn get_cc_parsing_context(&mut self, address: &CCAddress) -> CCParsingContext {
        CCParsingContext::builder()
            .source_node_id(address.source_node_id)
            .frame_addressing(Some((&address.destination).into()))
            .own_node_id(self.driver_api.own_node_id())
            .security_manager(self.driver_api.security_manager())
            .build()
    }

    async fn handle_frame(&mut self, frame: SerialFrame) {
        match frame {
            SerialFrame::ControlFlow(cf) => {
                // If the awaited control-flow-frame registry has a matching awaiter,
                // remove it and send the frame through its channel
                if let Some(channel) = self.awaited_control_flow_frames.take_matching(&cf) {
                    channel
                        .send(cf)
                        .expect("invoking the callback of an Awaited should not fail");
                    return;
                }
            }

            SerialFrame::Command(raw) => {
                // Try to convert it into an actual command
                let cmd = {
                    let ctx = self.get_command_parsing_context();
                    match zwave_serial::command::Command::try_from_raw(raw, ctx) {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            println!("{} failed to decode CommandRaw: {}", now(), e);
                            // TODO: Handle misformatted frames
                            return;
                        }
                    }
                };

                // Log the received command
                let address = match &cmd {
                    Command::ApplicationCommandRequest(cmd) => Some(cmd.command.address()),
                    Command::BridgeApplicationCommandRequest(cmd) => Some(cmd.command.address()),
                    _ => None,
                };

                if let Some(address) = address {
                    self.driver_api
                        .node_log(address.source_node_id, address.endpoint_index)
                        .command(&cmd, Direction::Inbound);
                } else {
                    self.driver_api
                        .controller_log()
                        .command(&cmd, Direction::Inbound);
                }

                // If the awaited command registry has a matching awaiter,
                // remove it and send the command through its channel
                if let Some(channel) = self.awaited_commands.take_matching(&cmd) {
                    channel
                        .send(cmd.clone())
                        .expect("invoking the callback of an Awaited should not fail");
                    return;
                }

                match cmd {
                    // Handle the CC if there is one
                    Command::ApplicationCommandRequest(cmd) => {
                        self.handle_cc(cmd.command);
                        return;
                    }
                    Command::BridgeApplicationCommandRequest(cmd) => {
                        self.handle_cc(cmd.command);
                        return;
                    }

                    // Or handle all other commands
                    _ => {
                        println!("TODO: Handle command {:?}", &cmd);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_cc(&mut self, mut cc: WithAddress<CC>) {
        let node_logger = self
            .driver_api
            .node_log(cc.address().source_node_id, cc.address().endpoint_index);

        // Check if the CC is split across multiple partial CCs
        if let Some(session_id) = cc.session_id() {
            // If so, try to merge it
            let ctx = self.get_cc_parsing_context(cc.address());
            if let Err(e) = cc.merge_session(ctx, vec![]) {
                node_logger.error(|| format!("failed to merge partial CCs: {}", e));
                return;
            }
        }

        // FIXME: Check if low-security command needs to be discarded

        // Persist CC values. TODO: test first if we should
        let mut cache = ValueCache::new(&self.driver_api.storage);
        persist_cc_values(&cc, &mut cache);

        // If the awaited CC registry has a matching awaiter,
        // remove it and send the CC through its channel
        if let Some(channel) = self.awaited_ccs.take_matching(&cc) {
            channel
                .send(cc.clone())
                .expect("invoking the callback of an Awaited should not fail");

            return;
        }

        #[allow(clippy::single_match)]
        match cc.deref() {
            CC::SecurityCCNonceGet(nonce_get) => {
                self.handle_security_cc_nonce_get(nonce_get, cc.address());
            }
            _ => {}
        };

        // FIXME: Handle unsolicited CC
    }

    fn handle_security_cc_nonce_get(
        &mut self,
        _nonce_get: &SecurityCCNonceGet,
        address: &CCAddress,
    ) {
        let Some(mut sec_man) = self.driver_api.security_manager() else {
            return;
        };

        let node_id = address.source_node_id;

        sec_man.delete_nonce_for_receiver(node_id);
        let nonce = sec_man.generate_nonce(node_id);
        let nonce_id = nonce.id();
        let cc = SecurityCCNonceReport { nonce }.with_destination(node_id.into());

        let driver = self.driver_api.clone();
        tokio::spawn(async move {
            // FIXME: Set options: ACK | AutoRoute, 1 send attempt, immediate priority, don't change node status
            let send_result = driver.exec_node_command(&cc, None).await;
            if send_result.is_err() {
                // The nonce could not be sent, invalidate it
                sec_man.delete_own_nonce(nonce_id);
            }
        });
    }
}

impl BackgroundTask for MainLoop {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                // Make sure we don't read from the serial port if there is a potential command
                // waiting to set up a frame handler
                biased;

                // We received a shutdown signal
                _ = self.shutdown.notified() => break,

                // We received a command from the outside
                Some(cmd) = self.cmd_rx.recv() => self.handle_command(cmd).await,

                // The serial port emitted a frame
                Ok(frame) = self.serial_listener.recv() => self.handle_frame(frame).await
            }
        }

        eprintln!("MainLoop shutting down");
    }
}

fn persist_cc_values(cc: &WithAddress<CC>, cache: &mut ValueCache) {
    // FIXME: Recurse through encapsulation CCs
    cache.write_many(cc.to_values().into_iter().map(|(value_id, value)| {
        let value_id = EndpointValueId::new(
            cc.address().source_node_id,
            cc.address().endpoint_index,
            value_id,
        );
        println!("Persisting {:?} = {:?}", value_id, value);
        (value_id, value)
    }));
}
