use crate::{Direction, LocalImmutableLogger, LogInfo, Logger};
use zwave_core::log::{LogPayload, Loglevel};
use zwave_serial::frame::ControlFlow;

pub struct SerialLogger2<'a> {
    inner: &'a dyn LocalImmutableLogger,
}

const SERIAL_LOGLEVEL: Loglevel = Loglevel::Debug;

impl<'a> SerialLogger2<'a> {
    pub fn new(inner: &'a dyn LocalImmutableLogger) -> Self {
        Self { inner }
    }

    pub fn data(&self, data: &[u8], direction: Direction) {
        if self.inner.log_level() < SERIAL_LOGLEVEL {
            return;
        }

        let message = format!("0x{}", hex::encode(data));
        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(direction)
            .secondary_tag(format!("{} bytes", data.len()).into())
            .payload(LogPayload::Text(message.into()))
            .build();
        self.inner.log(log, SERIAL_LOGLEVEL);
    }

    pub fn control_flow(&self, byte: ControlFlow, direction: Direction) {
        if self.inner.log_level() < SERIAL_LOGLEVEL {
            return;
        }

        let tag = format!("{:#04x}", byte as u8).into();

        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(direction)
            .primary_tags(vec![byte.to_string().into()])
            .secondary_tag(tag)
            .payload(LogPayload::empty())
            .build();
        self.inner.log(log, SERIAL_LOGLEVEL);
    }

    pub fn discarded(&self, data: &[u8]) {
        if self.inner.log_level() < SERIAL_LOGLEVEL {
            return;
        }

        let message = format!("invalid data: 0x{}", hex::encode(data));
        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(Direction::Inbound)
            .primary_tags(vec!["DISCARDED".into()])
            .secondary_tag(format!("{} bytes", data.len()).into())
            .payload(LogPayload::Text(message.into()))
            .build();
        self.inner.log(log, SERIAL_LOGLEVEL);
    }
}
