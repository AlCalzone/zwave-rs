use custom_debug_derive::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Serialport(#[from] serialport::Error),
    #[error("Parser error: {0:?}")]
    Parser(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Provides a way to convert custom results into this library's result type.
pub trait IntoResult {
    type Output;
    fn into_result(self) -> Result<Self::Output>;
}

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum ParserErrorCode {
//     Format,
//     #[debug("Checksum mismatch: expected {expected:#04x}, got {actual:#04x}")]
//     Checksum {
//         expected: u8,
//         actual: u8,
//     },
// }

// impl<I> nom::ErrorConvert<Error> for parse::Error<I> {
//     fn convert(self) -> Error {
//         Error::Parser(ParserErrorCode::Format)
//     }
// }

// impl<I> From<nom::Err<I>> for Error {
//     fn from(value: nom::Err<I>) -> Self {
//         Error::Parser(ParserErrorCode::Format)
//     }
// }
