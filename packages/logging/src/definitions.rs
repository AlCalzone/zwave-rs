use crate::util::str_width;
use chrono::{DateTime, Utc};
use std::{borrow::Cow, sync::OnceLock};
use termcolor::ColorSpec;
use typed_builder::TypedBuilder;

const NESTED_INDENT: usize = 2;
fn nested_indent_str() -> &'static str {
    static STR: OnceLock<String> = OnceLock::new();
    STR.get_or_init(|| " ".repeat(NESTED_INDENT))
}

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

pub trait FlattenLog {
    fn flatten_log(&self) -> Vec<Cow<'static, str>>;
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

#[derive(Clone, TypedBuilder)]
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

#[derive(Clone)]
pub enum LogPayload {
    Text(LogPayloadText),
    Dict(LogPayloadDict),
    Flat(Vec<Cow<'static, str>>),
}

impl FlattenLog for LogPayload {
    fn flatten_log(&self) -> Vec<Cow<'static, str>> {
        match self {
            LogPayload::Text(text) => text.flatten_log(),
            LogPayload::Dict(dict) => dict.flatten_log(),
            LogPayload::Flat(lines) => lines.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LogPayloadText {
    pub lines: Vec<Cow<'static, str>>,
    pub nested: Option<Box<LogPayload>>,
}

impl FlattenLog for LogPayloadText {
    fn flatten_log(&self) -> Vec<Cow<'static, str>> {
        let mut ret = self.lines.clone();
        if let Some(nested) = &self.nested {
            ret.extend(
                nested
                    .as_ref()
                    .flatten_log()
                    .iter()
                    .map(|item| format!("{}{}", nested_indent_str(), item).into()),
            );
        }

        ret
    }
}

#[derive(Clone)]
pub struct LogPayloadDict {
    pub entries: Vec<(Cow<'static, str>, LogPayloadDictValue)>,
    pub nested: Option<Box<LogPayload>>,
}

impl FlattenLog for LogPayloadDict {
    fn flatten_log(&self) -> Vec<Cow<'static, str>> {
        // Dicts align their values by the longest key, so we have to iterate twice
        let max_key_width = self
            .entries
            .iter()
            .filter_map(|(key, value)| match value {
                LogPayloadDictValue::Text(_) => Some(str_width(key)),
                LogPayloadDictValue::List(_) => None,
            })
            .max()
            .unwrap_or(0);

        let mut ret = Vec::new();
        // Add the dict itself
        for (key, value) in self.entries.iter() {
            match value {
                // Text values have the key and value on the same line
                LogPayloadDictValue::Text(text) => {
                    ret.push(
                        format!(
                            "{:width$} {}",
                            format!("{}:", key),
                            text,
                            width = max_key_width + 1
                        )
                        .into(),
                    );
                }
                // Lists are on the next line after the key and indented
                LogPayloadDictValue::List(list) => {
                    ret.push(format!("{}:", key).into());
                    ret.extend(
                        list.flatten_log()
                            .iter()
                            .map(|item| format!("{}{}", nested_indent_str(), item).into()),
                    );
                }
            }
        }
        // Then append the nested payload, indented
        if let Some(nested) = &self.nested {
            ret.extend(
                nested
                    .as_ref()
                    .flatten_log()
                    .iter()
                    .map(|item| format!("{}{}", nested_indent_str(), item).into()),
            );
        }
        ret
    }
}

#[derive(Clone)]
pub enum LogPayloadDictValue {
    Text(Cow<'static, str>),
    List(LogList),
}

#[derive(Clone)]
pub struct LogList {
    pub bullet: &'static str,
    pub items: Vec<Cow<'static, str>>,
}

impl FlattenLog for LogList {
    fn flatten_log(&self) -> Vec<Cow<'static, str>> {
        self.items
            .iter()
            .map(|item| format!("{} {}", self.bullet, item).into())
            .collect()
    }
}
