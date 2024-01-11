use crate::{ImmutableLogger, LogInfo, Loglevel};
use std::{borrow::Cow, sync::Arc};

pub struct DriverLogger {
    inner: Arc<dyn ImmutableLogger>,
}

const LOGO: &str = "\
🦀🦀🦀       🦀    🦀   🦀🦀🦀   🦀    🦀  🦀🦀🦀       🦀🦀🦀     🦀🦀🦀
   🦀        🦀    🦀  🦀    🦀  🦀    🦀  🦀           🦀   🦀   🦀
  🦀   🦀🦀  🦀 🦀 🦀  🦀🦀🦀🦀  🦀    🦀  🦀🦀         🦀🦀🦀     🦀🦀🦀
 🦀          🦀 🦀 🦀  🦀    🦀   🦀  🦀   🦀           🦀   🦀         🦀
🦀🦀🦀        🦀  🦀   🦀    🦀    🦀🦀    🦀🦀🦀       🦀    🦀   🦀🦀🦀\
";

impl DriverLogger {
    pub fn new(inner: Arc<dyn ImmutableLogger>) -> Self {
        Self { inner }
    }

    pub fn logo(&self) {
        self.message(LOGO);
    }

    pub fn message(&self, message: impl Into<Cow<'static, str>>) {
        let message_lines: Vec<_> = String::from(message.into())
            .split('\n')
            .map(|s| s.to_owned().into())
            .collect();
        let log = LogInfo::builder()
            .label("DRIVER")
            .payload(crate::LogPayload {
                message_lines: Some(message_lines),
                payload: None,
            })
            .build();
        self.inner.log(log, Loglevel::Info);
    }
}
