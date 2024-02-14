use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Needed {
    Unknown,
    Size(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorContext {
    None,
    String(&'static str),
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorContext::None => write!(f, "No context"),
            ErrorContext::String(s) => write!(f, "{}", s),
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
        ErrorContext::String(s)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Incomplete data: {0:?} bytes needed")]
    Incomplete(Needed),
    #[error("Recoverable error: {0}")]
    Recoverable(ErrorContext),
    #[error("Final error: {0}")]
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
}

pub type ParseResult<O> = Result<O, ParseError>;
