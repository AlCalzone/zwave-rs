use crate::{Direction, LocalImmutableLogger, LogInfo, Logger};
use std::borrow::Cow;
use zwave_core::{
    definitions::*,
    log::{LogPayload, LogPayloadText, Loglevel},
};
use zwave_serial::command::CommandId;

pub struct NodeLogger<'a> {
    node_id: NodeId,
    endpoint: EndpointIndex,
    inner: &'a dyn LocalImmutableLogger,
}

impl<'a> NodeLogger<'a> {
    pub fn new(inner: &'a dyn LocalImmutableLogger, node_id: NodeId, endpoint: EndpointIndex) -> Self {
        Self {
            inner,
            node_id,
            endpoint,
        }
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn message<L: Into<LogPayload>>(&self, message: impl Fn() -> L, level: Loglevel) {
        if self.inner.log_level() < level {
            return;
        }

        let mut primary_tags = vec![format!("Node {:0>3}", self.node_id).into()];
        if let EndpointIndex::Endpoint(index) = self.endpoint {
            primary_tags.push(format!("EP {}", index).into());
        }

        let log = LogInfo::builder()
            .label("CNTRLR")
            .primary_tags(primary_tags)
            .payload(message().into())
            .build();
        self.inner.log(log, level);
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn command(&self, command: &impl CommandId, direction: Direction) {
        let level = Loglevel::Debug;
        if self.inner.log_level() < level {
            return;
        }

        let node_id_tag = format!("Node {:0>3}", self.node_id);
        let mut primary_tags: Vec<Cow<_>> = vec![node_id_tag.into()];

        if let EndpointIndex::Endpoint(index) = self.endpoint {
            primary_tags.push(format!("EP {}", index).into());
        }

        let type_tag = if command.command_type() == CommandType::Request {
            "REQ"
        } else {
            "RES"
        };
        primary_tags.push(type_tag.into());

        let function_tag = format!("{:?}", command.function_type());
        primary_tags.push(function_tag.into());

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
