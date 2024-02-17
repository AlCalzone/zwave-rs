use crate::{ImmutableLogger, LogInfo};
use std::{borrow::Cow, sync::Arc};
use zwave_core::{
    log::{LogPayload, Loglevel},
    util::to_lines,
};

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
        self.info(|| LOGO);
    }

    // FIXME: Remove duplication with ControllerLogger
    pub fn message<L: Into<Cow<'static, str>>>(&self, message: impl Fn() -> L, level: Loglevel) {
        if self.level() < level {
            return;
        }

        let log = LogInfo::builder()
            .label("DRIVER")
            .payload(LogPayload::Flat(to_lines(message())))
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
