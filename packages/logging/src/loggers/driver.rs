use crate::{Direction, ImmutableLogger, LogInfo, Loglevel};
use std::{borrow::Cow, time::Instant, vec};

pub struct DriverLogger {
    inner: Box<dyn ImmutableLogger>,
}

impl DriverLogger {
    pub fn new(inner: Box<dyn ImmutableLogger>) -> Self {
        Self { inner }
    }

    fn log_level(&self) -> Loglevel {
        self.inner.log_level()
    }

    fn set_log_level(&self, level: Loglevel) {
        self.inner.set_log_level(level);
    }

    pub fn message(&self, message: impl Into<Cow<'static, str>>) {
        let info = LogInfo {
            timestamp: Instant::now(),
            direction: Direction::None,
            label: "DRIVER",
            primary_tags: None,
            secondary_tags: None,
            payload: crate::LogPayload {
                message_lines: Some(vec![message.into()]),
                payload: None,
            },
        };
        self.inner.log(info, Loglevel::Info);
    }
}
