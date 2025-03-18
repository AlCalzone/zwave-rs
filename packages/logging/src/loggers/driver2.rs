use crate::{LocalImmutableLogger, LogInfo, Logger};
use std::borrow::Cow;
use zwave_core::log::{LogPayload, Loglevel};

pub struct DriverLogger2<'a> {
    inner: &'a dyn LocalImmutableLogger,
}

const LOGO: &str = "\
🦀🦀🦀       🦀    🦀   🦀🦀🦀   🦀    🦀  🦀🦀🦀       🦀🦀🦀     🦀🦀🦀
   🦀        🦀    🦀  🦀    🦀  🦀    🦀  🦀           🦀   🦀   🦀
  🦀   🦀🦀  🦀 🦀 🦀  🦀🦀🦀🦀  🦀    🦀  🦀🦀         🦀🦀🦀     🦀🦀🦀
 🦀          🦀 🦀 🦀  🦀    🦀   🦀  🦀   🦀           🦀   🦀         🦀
🦀🦀🦀        🦀  🦀   🦀    🦀    🦀🦀    🦀🦀🦀       🦀    🦀   🦀🦀🦀\
";

impl<'a> DriverLogger2<'a> {
    pub fn new(inner: &'a dyn LocalImmutableLogger) -> Self {
        Self { inner }
    }

    pub fn logo(&self) {
        self.info(|| LOGO);
    }

    // FIXME: Remove duplication with ControllerLogger
    pub fn message<L: Into<Cow<'static, str>>>(
        &self,
        message: impl Fn() -> L,
        level: Loglevel,
    ) {
        if self.level() < level {
            return;
        }

        let message: Cow<'static, str> = message().into();
        let log = LogInfo::builder()
            .label("DRIVER")
            .payload(LogPayload::Text(message.into()))
            .build();
        self.inner.log(log, level);
    }

    pub fn error<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Error);
    }

    pub fn warn<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Warn);
    }

    pub fn info<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Info);
    }

    pub fn verbose<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Verbose);
    }

    pub fn debug<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Debug);
    }

    pub fn silly<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L) {
        self.message(message, Loglevel::Silly);
    }

    pub fn level(&self) -> Loglevel {
        self.inner.log_level()
    }
}
