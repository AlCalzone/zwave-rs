use crate::{ImmutableLogger, LogInfo, Loglevel};
use std::{borrow::Cow, sync::Arc};
use zwave_core::{definitions::*, log::{LogPayload, ToLogPayload}, util::to_lines};

pub struct ControllerLogger {
    inner: Arc<dyn ImmutableLogger>,
}

impl ControllerLogger {
    pub fn new(inner: Arc<dyn ImmutableLogger>) -> Self {
        Self { inner }
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn message(&self, message: impl Into<LogPayload>, level: Loglevel) {
        let log = LogInfo::builder()
            .label("CNTRLR")
            .payload(message.into())
            .build();
        self.inner.log(log, level);
    }

    pub fn error(&self, message: impl Into<LogPayload>) {
        self.message(message, Loglevel::Error);
    }

    pub fn warn(&self, message: impl Into<LogPayload>) {
        self.message(message, Loglevel::Warn);
    }

    pub fn info(&self, message: impl Into<LogPayload>) {
        self.message(message, Loglevel::Info);
    }

    pub fn verbose(&self, message: impl Into<LogPayload>) {
        self.message(message, Loglevel::Verbose);
    }

    pub fn debug(&self, message: impl Into<LogPayload>) {
        self.message(message, Loglevel::Debug);
    }

    pub fn silly(&self, message: impl Into<LogPayload>) {
        self.message(message, Loglevel::Silly);
    }
}
