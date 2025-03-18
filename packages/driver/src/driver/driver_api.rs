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

    // FIXME: Assert that the driver is ready for this command
    pub async fn exec_controller_command<C>(
        &self,
        command: C,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ExecControllerCommandResult<Option<Command>>
    where
        C: ExecutableCommand + 'static,
    {
        // FIXME:
        // let options = match options {
        //     Some(options) => options.clone(),
        //     None => Default::default(),
        // };

        // let supported = self.supports_function(command.function_type());
        // if options.enforce_support && !supported {
        //     return Err(ExecControllerCommandError::Unsupported(format!(
        //         "{:?}",
        //         command.function_type()
        //     )));
        // }

        let result = self.execute_serial_api_command(command).await;
        // TODO: Handle retrying etc.
        match result {
            Ok(SerialApiMachineResult::Success(command)) => Ok(command),
            Ok(result) => Err(result.into()),
            Err(e) => Err(ExecControllerCommandError::Unexpected(format!(
                "unexpected error in execute_serial_api_command: {:?}",
                e
            ))),
        }
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
