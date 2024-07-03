use super::{
    awaited::{AwaitedRef, AwaitedRegistry, Predicate},
    SerialListener, TaskCommandReceiver, TaskCommandSender,
};
use crate::{
    cache::ValueCache, define_async_task_commands, driver_api::DriverApiImpl, BackgroundTask,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;
use zwave_cc::commandclass::{CCParsingContext, CCSession, CCValues, WithAddress, CC};
use zwave_core::{
    cache::Cache, util::now, value_id::EndpointValueId, wrapping_counter::WrappingCounter,
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
    SetDriverApi -> () {
        driver_api: Box<dyn DriverApiImpl>
    }
});

pub(crate) type MainTaskCommandSender = TaskCommandSender<MainTaskCommand>;
pub(crate) type MainTaskCommandReceiver = TaskCommandReceiver<MainTaskCommand>;

struct MainLoopStorage {
    awaited_control_flow_frames: Arc<AwaitedRegistry<ControlFlow>>,
    awaited_commands: Arc<AwaitedRegistry<Command>>,
    awaited_ccs: Arc<AwaitedRegistry<WithAddress<CC>>>,
    callback_id_gen: WrappingCounter<u8>,
}

pub struct MainLoop {
    driver_api: Box<dyn DriverApiImpl>,
    storage: MainLoopStorage,

    shutdown: Arc<Notify>,
    cmd_rx: MainTaskCommandReceiver,
    serial_listener: SerialListener,
}

impl MainLoop {
    pub fn new(
        driver_api: Box<dyn DriverApiImpl>,
        shutdown: Arc<Notify>,
        cmd_rx: MainTaskCommandReceiver,
        serial_listener: SerialListener,
    ) -> Self {
        let storage = MainLoopStorage {
            awaited_control_flow_frames: Arc::new(AwaitedRegistry::default()),
            awaited_commands: Arc::new(AwaitedRegistry::default()),
            awaited_ccs: Arc::new(AwaitedRegistry::default()),
            callback_id_gen: WrappingCounter::new(),
        };

        Self {
            driver_api,
            storage,
            shutdown,
            cmd_rx,
            serial_listener,
        }
    }

    async fn handle_command(&mut self, cmd: MainTaskCommand) {
        match cmd {
            MainTaskCommand::RegisterAwaitedControlFlowFrame(ctrl) => {
                let result = self
                    .storage
                    .awaited_control_flow_frames
                    .add(ctrl.predicate, ctrl.timeout);
                ctrl.callback
                    .send(result)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::RegisterAwaitedCommand(cmd) => {
                let result = self
                    .storage
                    .awaited_commands
                    .add(cmd.predicate, cmd.timeout);
                cmd.callback
                    .send(result)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::RegisterAwaitedCC(cc) => {
                let result = self.storage.awaited_ccs.add(cc.predicate, cc.timeout);
                cc.callback
                    .send(result)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::GetNextCallbackId(cmd) => {
                let id = self.storage.callback_id_gen.increment();
                cmd.callback
                    .send(id)
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }

            MainTaskCommand::SetDriverApi(cmd) => {
                self.driver_api = cmd.driver_api;
                cmd.callback
                    .send(())
                    .expect("invoking the callback of a MainTaskCommand should not fail");
            }
        }
    }

    async fn handle_frame(&mut self, frame: SerialFrame) {
        let driver = &self.driver_api;

        match frame {
            SerialFrame::ControlFlow(cf) => {
                // If the awaited control-flow-frame registry has a matching awaiter,
                // remove it and send the frame through its channel
                if let Some(channel) = self.storage.awaited_control_flow_frames.take_matching(&cf) {
                    channel
                        .send(cf)
                        .expect("invoking the callback of an Awaited should not fail");
                    return;
                }
            }

            SerialFrame::Command(raw) => {
                // Try to convert it into an actual command
                let ctx = CommandParsingContext::builder()
                    .own_node_id(driver.storage().own_node_id())
                    .node_id_type(driver.storage().node_id_type())
                    .sdk_version(driver.storage().sdk_version())
                    .security_manager(driver.security_manager())
                    .build();
                let cmd = match zwave_serial::command::Command::try_from_raw(raw, &ctx) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        println!("{} failed to decode CommandRaw: {}", now(), e);
                        // TODO: Handle misformatted frames
                        return;
                    }
                };

                // Log the received command
                let address = match &cmd {
                    Command::ApplicationCommandRequest(cmd) => Some(cmd.command.address()),
                    Command::BridgeApplicationCommandRequest(cmd) => Some(cmd.command.address()),
                    _ => None,
                };

                if let Some(address) = address {
                    driver
                        .node_log(address.source_node_id, address.endpoint_index)
                        .command(&cmd, Direction::Inbound);
                } else {
                    driver.controller_log().command(&cmd, Direction::Inbound);
                }

                // If the awaited command registry has a matching awaiter,
                // remove it and send the command through its channel
                if let Some(channel) = self.storage.awaited_commands.take_matching(&cmd) {
                    channel
                        .send(cmd.clone())
                        .expect("invoking the callback of an Awaited should not fail");
                    return;
                }

                match cmd {
                    // Handle the CC if there is one
                    Command::ApplicationCommandRequest(cmd) => {
                        let ctx = cmd.get_cc_parsing_context(&ctx);
                        let cc = cmd.command;
                        self.handle_cc(cc, ctx);
                        return;
                    }
                    Command::BridgeApplicationCommandRequest(cmd) => {
                        let ctx = cmd.get_cc_parsing_context(&ctx);
                        let cc = cmd.command;
                        self.handle_cc(cc, ctx);
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

    fn handle_cc(&self, mut cc: WithAddress<CC>, ctx: CCParsingContext) {
        let driver = &self.driver_api;

        let node_logger = driver.node_log(cc.address().source_node_id, cc.address().endpoint_index);

        // Check if the CC is split across multiple partial CCs
        if let Some(session_id) = cc.session_id() {
            // If so, try to merge it
            if let Err(e) = cc.merge_session(&ctx, vec![]) {
                node_logger.error(|| format!("failed to merge partial CCs: {}", e));
                return;
            }
        }

        // FIXME: Check if low-security command needs to be discarded

        // Persist CC values. TODO: test first if we should
        let mut cache = ValueCache::new(driver.storage());
        persist_cc_values(&cc, &mut cache);

        // If the awaited CC registry has a matching awaiter,
        // remove it and send the CC through its channel
        if let Some(channel) = self.storage.awaited_ccs.take_matching(&cc) {
            channel
                .send(cc.clone())
                .expect("invoking the callback of an Awaited should not fail");

            return;
        }

        // FIXME: Handle unsolicited CC
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
