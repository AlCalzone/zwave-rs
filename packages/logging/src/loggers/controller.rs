use crate::{ImmutableLogger, LogInfo, Loglevel, LogPayload};
use std::{borrow::Cow, sync::Arc};

pub struct ControllerLogger {
    inner: Arc<dyn ImmutableLogger>,
}

impl ControllerLogger {
    pub fn new(inner: Arc<dyn ImmutableLogger>) -> Self {
        Self { inner }
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn message(&self, message: impl Into<Cow<'static, str>>, level: Loglevel) {
        let message_lines: Vec<_> = String::from(message.into())
            .split('\n')
            .map(|s| s.to_owned().into())
            .collect();
        let log = LogInfo::builder()
            .label("CNTRLR")
            .payload(LogPayload::Flat(message_lines))
            .build();
        self.inner.log(log, level);
    }

    pub fn error(&self, message: impl Into<Cow<'static, str>>) {
        self.message(message, Loglevel::Error);
    }

    pub fn warn(&self, message: impl Into<Cow<'static, str>>) {
        self.message(message, Loglevel::Warn);
    }

    pub fn info(&self, message: impl Into<Cow<'static, str>>) {
        self.message(message, Loglevel::Info);
    }

    pub fn verbose(&self, message: impl Into<Cow<'static, str>>) {
        self.message(message, Loglevel::Verbose);
    }

    pub fn debug(&self, message: impl Into<Cow<'static, str>>) {
        self.message(message, Loglevel::Debug);
    }

    pub fn silly(&self, message: impl Into<Cow<'static, str>>) {
        self.message(message, Loglevel::Silly);
    }
}
