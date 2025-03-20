use super::{awaited::Predicate, Driver, DriverInput};
use crate::{error::Result, ExecutableCommand, SerialApiMachineResult};
use futures::channel::oneshot;
use std::time::Duration;
use zwave_cc::prelude::*;
use zwave_core::log::Loglevel;
use zwave_core::prelude::*;
use zwave_logging::{
    loggers::{controller::ControllerLogger, driver::DriverLogger, node::NodeLogger},
    LocalImmutableLogger, LogInfo,
};

impl Driver {
    pub(crate) fn driver_log(&self) -> DriverLogger {
        DriverLogger::new(self)
    }

    pub(crate) fn controller_log(&self) -> ControllerLogger {
        ControllerLogger::new(self)
    }

    pub(crate) fn node_log(&self, node_id: NodeId, endpoint: EndpointIndex) -> NodeLogger {
        NodeLogger::new(self, node_id, endpoint)
    }

    fn dispatch(&self, input: DriverInput) {
        self.cmd_tx
            .clone()
            .try_send(input)
            .expect("Failed to dispatch command");
    }

    pub(crate) fn init_security_managers(&self) {
        self.dispatch(DriverInput::InitSecurityManagers);
    }

    pub async fn await_cc(
        &self,
        predicate: Predicate<WithAddress<CC>>,
        timeout: Option<Duration>,
    ) -> Result<WithAddress<CC>> {
        let (tx, rx) = oneshot::channel();
        let cmd = DriverInput::AwaitCC {
            predicate,
            timeout,
            callback: tx,
        };
        self.dispatch(cmd);

        rx.await.expect("Failed to receive callback for await_cc")
    }
}

impl LocalImmutableLogger for Driver {
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
