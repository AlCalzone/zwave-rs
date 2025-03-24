use crate::ControllerCommandError;
use thiserror::Error;
use zwave_serial::error::Error as SerialPortError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The driver is not ready")]
    NotReady,
    #[error(transparent)]
    SerialPort(#[from] SerialPortError),
    #[error(transparent)]
    Controller(#[from] ControllerCommandError),
    #[error("Internal error")]
    Internal,
    #[error("Operation timed out")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;
