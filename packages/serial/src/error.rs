use custom_debug_derive::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Serialport(#[from] serialport::Error),
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Provides a way to convert custom results into this library's result type
/// without breaking the orphan rule
pub trait IntoResult {
    type Output;
    fn into_result(self) -> Result<Self::Output>;
}
