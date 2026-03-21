#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(feature = "std")]
    #[error(transparent)]
    StdIo(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

/// Provides a way to convert custom results into this library's result type
/// without breaking the orphan rule
pub trait IntoResult {
    type Output;
    fn into_result(self) -> Result<Self::Output>;
}
