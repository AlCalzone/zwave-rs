use crate::{Direction, ImmutableLogger, LogInfo};
use std::{borrow::Cow, sync::Arc};
use zwave_core::{
    definitions::*,
    log::{LogPayload, LogPayloadText, Loglevel, ToLogPayload},
};
use zwave_serial::command::{Command, CommandId};

pub struct ControllerLogger {
    inner: Arc<dyn ImmutableLogger>,
}

impl ControllerLogger {
    pub fn new(inner: Arc<dyn ImmutableLogger>) -> Self {
        Self { inner }
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn message<L: Into<LogPayload>>(&self, message: impl Fn() -> L, level: Loglevel) {
        if self.level() < level {
            return;
        }

        let log = LogInfo::builder()
            .label("CNTRLR")
            .payload(message().into())
            .build();
        self.inner.log(log, level);
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn command(&self, command: &Command, direction: Direction) {
        let level = Loglevel::Debug;
        if self.level() < level {
            return;
        }

        let type_tag = if command.command_type() == CommandType::Request {
            "REQ"
        } else {
            "RES"
        };
        let function_tag = format!("{:?}", command.function_type());
        let primary_tags: Vec<Cow<_>> = vec![type_tag.into(), function_tag.into()];

        let payload = LogPayloadText::new("").with_nested(command.to_log_payload());

        let log = LogInfo::builder()
            .label("CNTRLR")
            .primary_tags(primary_tags)
            .direction(direction)
            .payload(payload.into())
            .build();
        self.inner.log(log, level);
    }

    pub fn error<L: Into<LogPayload>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Error);
    }

    pub fn warn<L: Into<LogPayload>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Warn);
    }

    pub fn info<L: Into<LogPayload>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Info);
    }

    pub fn verbose<L: Into<LogPayload>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Verbose);
    }

    pub fn debug<L: Into<LogPayload>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Debug);
    }

    pub fn silly<L: Into<LogPayload>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Silly);
    }

    pub fn level(&self) -> Loglevel {
        self.inner.log_level()
    }
}
