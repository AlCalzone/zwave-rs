use super::serial_api_machine::SerialApiMachineResult;
use super::{ExecutableCommand, SerialApi, SerialApiInput};
use crate::error::Result;
use futures::channel::oneshot;
use zwave_core::log::Loglevel;
use zwave_core::prelude::*;
use zwave_logging::{LocalImmutableLogger, LogInfo};

impl SerialApi {
    // pub(crate) fn controller_log(&self) -> ControllerLogger {
    //     ControllerLogger::new(self)
    // }

    // pub(crate) fn node_log(&self, node_id: NodeId, endpoint: EndpointIndex) -> NodeLogger {
    //     NodeLogger::new(self, node_id, endpoint)
    // }

    fn dispatch(&self, input: SerialApiInput) {
        self.input_tx
            .clone()
            .try_send(input)
            .expect("Failed to dispatch command");
    }

    pub async fn execute_serial_api_command<C>(&self, command: C) -> Result<SerialApiMachineResult>
    where
        C: ExecutableCommand + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let cmd = SerialApiInput::ExecCommand {
            command: Box::new(command),
            callback: tx,
        };
        self.dispatch(cmd);

        rx.await.expect("Failed to receive command result")
    }

}

impl LocalImmutableLogger for SerialApi {
    fn log(&self, log: LogInfo, level: Loglevel) {
        self.dispatch(SerialApiInput::Log { log, level });
    }

    fn log_level(&self) -> Loglevel {
        Loglevel::Debug
    }

    fn set_log_level(&self, level: Loglevel) {
        todo!()
    }
}
