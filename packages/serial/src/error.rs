use zwave_pal::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(String),
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

/// Provides a way to convert custom results into this library's result type
/// without breaking the orphan rule
pub trait IntoResult {
    type Output;
    fn into_result(self) -> Result<Self::Output>;
}
