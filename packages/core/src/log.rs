use crate::util::{str_width, to_lines};
use std::{borrow::Cow, convert::From, fmt::Display, sync::OnceLock};

const NESTED_INDENT: usize = 2;
fn nested_indent_str() -> &'static str {
    static STR: OnceLock<String> = OnceLock::new();
    STR.get_or_init(|| " ".repeat(NESTED_INDENT))
}

pub trait ToLogPayload {
    fn to_log_payload(&self) -> LogPayload;
}

impl ToLogPayload for String {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::Flat(to_lines(self.to_owned()))
    }
}

impl ToLogPayload for &'static str {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::Flat(to_lines(*self))
    }
}

pub trait FlattenLog {
    fn flatten_log(&self) -> Vec<Cow<'static, str>>;
}

#[derive(Clone)]
pub enum LogPayload {
    Text(LogPayloadText),
    Dict(LogPayloadDict),
    List(LogPayloadList),
    Flat(Vec<Cow<'static, str>>),
}

impl LogPayload {
    pub fn empty() -> Self {
        Self::Flat(Vec::new())
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
        Self::Flat(lines)
    }
}

impl FlattenLog for LogPayload {
    fn flatten_log(&self) -> Vec<Cow<'static, str>> {
        match self {
            LogPayload::Text(text) => text.flatten_log(),
            LogPayload::Dict(dict) => dict.flatten_log(),
            LogPayload::List(list) => list.flatten_log(),
            LogPayload::Flat(lines) => lines.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LogPayloadText {
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
            lines: to_lines(text),
            nested: None,
        }
    }

    pub fn with_nested(mut self, nested: impl Into<LogPayload>) -> Self {
        self.nested = Some(Box::new(nested.into()));
        self
    }
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

impl FlattenLog for LogPayloadList {
    fn flatten_log(&self) -> Vec<Cow<'static, str>> {
        self.items
            .iter()
            .map(|item| format!("· {}", item).into())
            .collect()
    }
}
