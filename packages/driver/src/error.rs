use custom_debug_derive::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The driver is not ready")]
    NotReady,
    #[error("Internal error")]
    Internal,
    #[error("Operation timed out")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;
