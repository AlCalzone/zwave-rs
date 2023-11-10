use custom_debug_derive::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error")]
    Internal,
}

pub type Result<T> = std::result::Result<T, Error>;
