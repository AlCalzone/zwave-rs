use crate::{Direction, ImmutableLogger, LogInfo};
use std::{borrow::Cow, sync::Arc};
use zwave_core::{
    definitions::*,
    log::{LogPayload, LogPayloadText, ToLogPayload, Loglevel},
};
use zwave_serial::command::{Command, CommandId};

pub struct NodeLogger {
    node_id: NodeId,
    endpoint: EndpointIndex,
    inner: Arc<dyn ImmutableLogger>,
}

impl NodeLogger {
    pub fn new(inner: Arc<dyn ImmutableLogger>, node_id: NodeId, endpoint: EndpointIndex) -> Self {
        Self {
            inner,
            node_id,
            endpoint,
        }
    }

    // FIXME: Remove duplication with DriverLogger
    pub fn message(&self, message: impl Into<LogPayload>, level: Loglevel) {
        let mut primary_tags = vec![format!("Node {:0>3}", self.node_id).into()];
        if let EndpointIndex::Endpoint(index) = self.endpoint {
            primary_tags.push(format!("EP {}", index).into());
        }

        let log = LogInfo::builder()
            .label("CNTRLR")
            .primary_tags(primary_tags)
            .payload(message.into())
            .build();
        self.inner.log(log, level);
    }

    // FIXME: Remove duplication with ControllerLogger
    pub fn command(&self, command: &Command, direction: Direction) {
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
        self.inner.log(log, Loglevel::Debug);
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
