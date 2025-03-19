use std::sync::Arc;

use super::serial_api_machine::SerialApiMachineResult;
use super::storage::DriverStorage;
use super::{DriverInput, ExecutableCommand};
use crate::error::Result;
use crate::{
    ExecControllerCommandError, ExecControllerCommandOptions, ExecControllerCommandResult,
};
use futures::channel::{mpsc, oneshot};
use zwave_core::log::Loglevel;
use zwave_core::prelude::{EndpointIndex, NodeId};
use zwave_logging::loggers::controller2::ControllerLogger2;
use zwave_logging::loggers::node2::NodeLogger2;
use zwave_logging::{LocalImmutableLogger, LogInfo};
use zwave_serial::prelude::Command;

#[derive(Clone)]
pub struct DriverApi {
    cmd_tx: mpsc::Sender<DriverInput>,
    pub(crate) storage: Arc<DriverStorage>,
}

impl DriverApi {
    pub fn new(cmd_tx: mpsc::Sender<DriverInput>, storage: Arc<DriverStorage>) -> Self {
        Self { cmd_tx, storage }
    }
    pub(crate) fn controller_log(&self) -> ControllerLogger2 {
        ControllerLogger2::new(self)
    }

    pub(crate) fn node_log(&self, node_id: NodeId, endpoint: EndpointIndex) -> NodeLogger2 {
        NodeLogger2::new(self, node_id, endpoint)
    }

    fn dispatch(&self, input: DriverInput) {
        self.cmd_tx
            .clone()
            .try_send(input)
            .expect("Failed to dispatch command");
    }

    pub async fn execute_serial_api_command<C>(&self, command: C) -> Result<SerialApiMachineResult>
    where
        C: ExecutableCommand + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let cmd = DriverInput::ExecCommand {
            command: Box::new(command),
            callback: tx,
        };
        self.dispatch(cmd);

        rx.await.expect("Failed to receive command result")
    }
}

impl LocalImmutableLogger for DriverApi {
    fn log(&self, log: LogInfo, level: Loglevel) {
        self.dispatch(DriverInput::Log { log, level });
    }

    fn log_level(&self) -> Loglevel {
        Loglevel::Debug
    }

    fn set_log_level(&self, level: Loglevel) {
        todo!()
    }
}
