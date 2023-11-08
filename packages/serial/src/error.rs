use cookie_factory::GenError;
use custom_debug_derive::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Serialport(#[from] serialport::Error),
    #[error("Parser error: {0:?}")]
    Parser(Option<String>),
    #[error("Serialization error: {0:?}")]
    Serialization(String),
}

impl From<GenError> for Error {
    fn from(e: GenError) -> Self {
        Error::Serialization(format!("{:?}", e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Provides a way to convert custom results into this library's result type
/// without breaking the orphan rule
pub trait IntoResult {
    type Output;
    fn into_result(self) -> Result<Self::Output>;
}
