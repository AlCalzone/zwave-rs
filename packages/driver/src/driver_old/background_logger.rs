use super::LogTaskCommandSender;
use crate::dispatch_oneshot;
use zwave_core::log::Loglevel;
use zwave_logging::{ImmutableLogger, LogInfo};

pub struct BackgroundLogger {
    cmd_tx: LogTaskCommandSender,
    level: Loglevel,
}

impl BackgroundLogger {
    pub(crate) fn new(cmd_tx: LogTaskCommandSender, level: Loglevel) -> Self {
        Self { cmd_tx, level }
    }
}

impl ImmutableLogger for BackgroundLogger {
    fn log(&self, log: LogInfo, level: Loglevel) {
        let _ = dispatch_oneshot!(self.cmd_tx, LogTaskCommand::Log, log, level);
    }

    fn log_level(&self) -> Loglevel {
        self.level
    }

    fn set_log_level(&self, level: Loglevel) {
        let _ = dispatch_oneshot!(self.cmd_tx, LogTaskCommand::UseLogLevel, level);
    }
}
