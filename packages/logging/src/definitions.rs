use chrono::{DateTime, Utc};
use std::borrow::Cow;
use termcolor::ColorSpec;
use typed_builder::TypedBuilder;

pub trait ToLogPayload {
    fn to_log_payload(&self) -> LogPayload;
}

pub trait LogFormatter {
    fn format_log(&self, log: &LogInfo, level: Loglevel) -> Vec<FormattedString>;
}

pub struct FormattedString {
    pub string: Cow<'static, str>,
    pub color: Option<ColorSpec>,
}

pub trait WithColor {
    fn with_color(self, color: ColorSpec) -> FormattedString;
}

impl FormattedString {
    pub fn new(string: impl Into<Cow<'static, str>>, color: Option<ColorSpec>) -> Self {
        Self {
            string: string.into(),
            color,
        }
    }
}

impl<T> From<T> for FormattedString
where
    T: Into<Cow<'static, str>>,
{
    fn from(string: T) -> Self {
        Self::new(string, None)
    }
}

impl<T> WithColor for T
where
    T: Into<Cow<'static, str>>,
{
    fn with_color(self, color: ColorSpec) -> FormattedString {
        FormattedString::new(self, Some(color))
    }
}

/// A trait for logging messages
pub trait Logger {
    fn log(&mut self, log: LogInfo, level: Loglevel);

    fn log_level(&self) -> Loglevel;
    fn set_log_level(&mut self, level: Loglevel);
}

/// A variant of the [Logger] trait that does not require mutability. This is typically an abstraction
/// over a message channel to another thread handling the actual logging.
pub trait ImmutableLogger: Send + Sync {
    fn log(&self, log: LogInfo, level: Loglevel);

    fn log_level(&self) -> Loglevel;
    fn set_log_level(&self, level: Loglevel);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Loglevel {
    Error,
    Warn,
    Info,
    Verbose,
    Debug,
    Silly,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    #[default]
    None,
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct LogInfo {
    #[builder(default = Utc::now())]
    pub timestamp: DateTime<Utc>,
    #[builder(default)]
    pub direction: Direction,
    pub label: &'static str,
    #[builder(default, setter(strip_option))]
    pub primary_tags: Option<Vec<Cow<'static, str>>>,
    #[builder(default, setter(strip_option))]
    pub secondary_tag: Option<Cow<'static, str>>,
    pub payload: LogPayload,
    // FIXME: Context
}

#[derive(Debug, Clone)]
pub struct LogPayload {
    pub message_lines: Option<Vec<Cow<'static, str>>>,
    pub payload: Option<Box<LogPayload>>,
}
