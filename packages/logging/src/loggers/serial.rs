use crate::{Direction, ImmutableLogger, LogInfo};
use std::sync::Arc;
use zwave_core::log::{LogPayload, Loglevel};
use zwave_serial::frame::ControlFlow;

pub struct SerialLogger {
    inner: Arc<dyn ImmutableLogger>,
}

impl SerialLogger {
    pub fn new(inner: Arc<dyn ImmutableLogger>) -> Self {
        Self { inner }
    }

    pub fn data(&self, data: &[u8], direction: Direction) {
        let message_lines: Vec<_> = vec![format!("0x{}", hex::encode(data)).into()];
        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(direction)
            .secondary_tag(format!("{} bytes", data.len()).into())
            .payload(LogPayload::Flat(message_lines))
            .build();
        self.inner.log(log, Loglevel::Debug);
    }

    pub fn control_flow(&self, byte: &ControlFlow, direction: Direction) {
        let tag = format!("{:#04x}", *byte as u8).into();

        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(direction)
            .primary_tags(vec![byte.to_string().into()])
            .secondary_tag(tag)
            .payload(LogPayload::Flat(Vec::new()))
            .build();
        self.inner.log(log, Loglevel::Debug);
    }

    pub fn discarded(&self, data: &[u8]) {
        let message_lines: Vec<_> = vec![format!("invalid data: 0x{}", hex::encode(data)).into()];
        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(Direction::Inbound)
            .primary_tags(vec!["DISCARDED".into()])
            .secondary_tag(format!("{} bytes", data.len()).into())
            .payload(LogPayload::Flat(message_lines))
            .build();
        self.inner.log(log, Loglevel::Debug);
    }
}
