use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error(transparent)]
    Serialport(#[from] serialport::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

