use crate::util::{str_width, to_lines};
use std::{borrow::Cow, convert::From};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Loglevel {
    Error,
    Warn,
    Info,
    Verbose,
    Debug,
    Silly,
}

pub trait ToLogPayload {
    fn to_log_payload(&self) -> LogPayload;
}

impl ToLogPayload for String {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::Text(self.to_owned().into())
    }
}

impl ToLogPayload for &'static str {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::Text((*self).into())
    }
}

pub trait NormalizeLogPayload {
    fn normalize(&self, indent_level: usize) -> NormalizedLogPayload;
}

#[derive(Clone)]
pub enum LogPayload {
    Empty,
    Text(LogPayloadText),
    Dict(LogPayloadDict),
    List(LogPayloadList),
}

impl LogPayload {
    pub fn empty() -> Self {
        Self::Empty
    }
}

impl<T> From<T> for LogPayload
where
    T: ToLogPayload,
{
    fn from(value: T) -> Self {
        value.to_log_payload()
    }
}

impl From<LogPayloadText> for LogPayload {
    fn from(text: LogPayloadText) -> Self {
        Self::Text(text)
    }
}

impl From<LogPayloadDict> for LogPayload {
    fn from(dict: LogPayloadDict) -> Self {
        Self::Dict(dict)
    }
}

impl From<LogPayloadList> for LogPayload {
    fn from(list: LogPayloadList) -> Self {
        Self::List(list)
    }
}

impl From<Vec<Cow<'static, str>>> for LogPayload {
    fn from(lines: Vec<Cow<'static, str>>) -> Self {
        LogPayloadText {
            tags: Vec::new(),
            lines,
            nested: None,
        }
        .into()
    }
}

impl NormalizeLogPayload for LogPayload {
    fn normalize(&self, indent_level: usize) -> NormalizedLogPayload {
        match self {
            LogPayload::Empty => NormalizedLogPayload::empty(indent_level),
            LogPayload::Text(text) => text.normalize(indent_level),
            LogPayload::Dict(dict) => dict.normalize(indent_level),
            LogPayload::List(list) => list.normalize(indent_level),
        }
    }
}

#[derive(Clone, Default)]
pub struct NormalizedLogPayload {
    pub indent_level: usize,
    pub tags: Vec<Cow<'static, str>>,
    pub lines: Vec<Cow<'static, str>>,
    pub nested: Option<Box<NormalizedLogPayload>>,
}

impl NormalizedLogPayload {
    pub fn empty(indent_level: usize) -> Self {
        Self {
            indent_level,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct LogPayloadText {
    pub tags: Vec<Cow<'static, str>>,
    pub lines: Vec<Cow<'static, str>>,
    pub nested: Option<Box<LogPayload>>,
}

impl<T> From<T> for LogPayloadText
where
    T: Into<Cow<'static, str>>,
{
    fn from(text: T) -> Self {
        Self::new(text)
    }
}

impl LogPayloadText {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tags: Vec::new(),
            lines: to_lines(text),
            nested: None,
        }
    }

    pub fn with_tag(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_nested(mut self, nested: impl Into<LogPayload>) -> Self {
        self.nested = Some(Box::new(nested.into()));
        self
    }
}

impl NormalizeLogPayload for LogPayloadText {
    fn normalize(&self, indent_level: usize) -> NormalizedLogPayload {
        NormalizedLogPayload {
            indent_level,
            tags: self.tags.clone(),
            lines: self.lines.clone(),
            nested: self
                .nested
                .as_ref()
                .map(|n| Box::new(n.normalize(indent_level + 1))),
        }
    }
}

#[derive(Default, Clone)]
pub struct LogPayloadDict {
    pub entries: Vec<(Cow<'static, str>, LogPayloadDictValue)>,
    pub nested: Option<Box<LogPayload>>,
}

impl LogPayloadDict {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            nested: None,
        }
    }

    pub fn with_entry(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<LogPayloadDictValue>,
    ) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    pub fn with_nested(mut self, nested: impl Into<LogPayload>) -> Self {
        self.nested = Some(Box::new(nested.into()));
        self
    }

    pub fn extend(mut self, other: LogPayloadDict) -> Self {
        self.entries.extend(other.entries);
        self
    }
}

impl<T> From<T> for LogPayloadDict
where
    T: Into<(Cow<'static, str>, LogPayloadDictValue)>,
{
    fn from(entry: T) -> Self {
        let mut ret = Self::new();
        ret.entries.push(entry.into());
        ret
    }
}

impl NormalizeLogPayload for LogPayloadDict {
    fn normalize(&self, indent_level: usize) -> NormalizedLogPayload {
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

        let mut lines = Vec::with_capacity(self.entries.len());
        // Add the dict itself
        for (key, value) in self.entries.iter() {
            match value {
                // Text values have the key and value on the same line
                LogPayloadDictValue::Text(text) => {
                    lines.push(
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
                    lines.push(format!("{}:", key).into());
                    lines.extend(list.normalize(indent_level + 1).lines);
                }
            }
        }

        NormalizedLogPayload {
            indent_level,
            tags: Vec::new(),
            lines,
            nested: self
                .nested
                .as_ref()
                // Nested items of a dict are not indented more than the dict itself for optical reasons
                .map(|n| Box::new(n.normalize(indent_level))),
        }
    }
}

#[derive(Clone)]
pub enum LogPayloadDictValue {
    Text(Cow<'static, str>),
    List(LogPayloadList),
}

macro_rules! impl_from_for_log_payload_dict_value {
    ($($type:ty),*) => {
        $(
            impl From<$type> for LogPayloadDictValue {
                fn from(value: $type) -> Self {
                    Self::Text(value.to_string().into())
                }
            }
        )*
    };
}

impl_from_for_log_payload_dict_value!(String, &'static str);
impl_from_for_log_payload_dict_value!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
impl_from_for_log_payload_dict_value!(bool);

impl<T> From<T> for LogPayloadDictValue
where
    T: Into<LogPayloadList>,
{
    fn from(list: T) -> Self {
        Self::List(list.into())
    }
}

#[derive(Clone)]
pub struct LogPayloadList {
    pub items: Vec<Cow<'static, str>>,
}

impl LogPayloadList {
    pub fn new(items: impl Iterator<Item = Cow<'static, str>>) -> Self {
        Self {
            items: items.collect(),
        }
    }
}

impl NormalizeLogPayload for LogPayloadList {
    fn normalize(&self, indent_level: usize) -> NormalizedLogPayload {
        let lines = self
            .items
            .iter()
            .map(|item| format!("· {}", item).into())
            .collect();

        NormalizedLogPayload {
            indent_level,
            tags: Vec::new(),
            lines,
            nested: None,
        }
    }
}
