use crate::{ImmutableLogger, LogInfo, Loglevel};
use std::{borrow::Cow, sync::Arc};
use zwave_core::definitions::*;

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
    pub fn message(&self, message: impl Into<Cow<'static, str>>, level: Loglevel) {
        let message_lines: Vec<_> = String::from(message.into())
            .split('\n')
            .map(|s| s.to_owned().into())
            .collect();
        let mut primary_tags = vec![format!("Node {:0>3}", self.node_id).into()];
        if let EndpointIndex::Endpoint(index) = self.endpoint {
            primary_tags.push(format!("EP {}", index).into());
        }

        let log = LogInfo::builder()
            .label("CNTRLR")
            .primary_tags(primary_tags)
            .payload(crate::LogPayload {
                message_lines: Some(message_lines),
                payload: None,
            })
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
