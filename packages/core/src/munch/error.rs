use crate::encoding::TryFromReprError;
use std::{
    borrow::Cow,
    fmt::{Debug, Display, Error},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Needed {
    Unknown,
    Size(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorContext {
    None,
    String(Cow<'static, str>),
    NotImplemented(Cow<'static, str>),
    Validation(Cow<'static, str>),
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorContext::None => write!(f, "No context"),
            ErrorContext::String(s)
            | ErrorContext::Validation(s)
            | ErrorContext::NotImplemented(s) => write!(f, "{}", s),
        }
    }
}

impl From<()> for ErrorContext {
    fn from(_: ()) -> Self {
        ErrorContext::None
    }
}

impl From<&'static str> for ErrorContext {
    fn from(s: &'static str) -> Self {
        ErrorContext::String(s.into())
    }
}

impl From<String> for ErrorContext {
    fn from(s: String) -> Self {
        ErrorContext::String(s.into())
    }
}

impl From<Cow<'static, str>> for ErrorContext {
    fn from(s: Cow<'static, str>) -> Self {
        ErrorContext::String(s)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Incomplete data: {0:?} bytes needed")]
    Incomplete(Needed),
    #[error("Recoverable error: {0}")]
    Recoverable(ErrorContext),
    #[error("{0}")]
    Final(ErrorContext),
}

impl ParseError {
    pub fn needed(n: usize) -> Self {
        ParseError::Incomplete(Needed::Size(n))
    }

    pub fn recoverable(ctx: impl Into<ErrorContext>) -> Self {
        ParseError::Recoverable(ctx.into())
    }

    pub fn final_error(ctx: impl Into<ErrorContext>) -> Self {
        ParseError::Final(ctx.into())
    }

    pub fn not_implemented(ctx: impl Into<Cow<'static, str>>) -> Self {
        ParseError::Final(ErrorContext::NotImplemented(ctx.into()))
    }

    pub fn validation_failure(ctx: impl Into<Cow<'static, str>>) -> Self {
        ParseError::Final(ErrorContext::Validation(ctx.into()))
    }

    pub fn context(&self) -> Option<ErrorContext> {
        match self {
            ParseError::Recoverable(ctx) | ParseError::Final(ctx) => Some(ctx.clone()),
            _ => None,
        }
    }
}

pub type ParseResult<O> = Result<O, ParseError>;

impl<T> From<TryFromReprError<T>> for ParseError
where
    T: Debug,
{
    fn from(_value: TryFromReprError<T>) -> Self {
        Self::recoverable(())
    }
}

/// Validates that the given condition is satisfied, otherwise results in a
/// nom Failure with the given error message.
pub fn validate(condition: bool, message: impl Into<Cow<'static, str>>) -> ParseResult<()> {
    if condition {
        Ok(())
    } else {
        Err(ParseError::recoverable(message.into()))
    }
}
