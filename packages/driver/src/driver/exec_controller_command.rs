use crate::Driver;
use crate::SerialApiMachineResult;

use thiserror::Error;
use zwave_serial::{
    command::{Command, CommandRequest},
    frame::SerialFrame,
};

pub struct ExecControllerCommandOptions {}

pub type ControllerCommandResult<T> = Result<T, ControllerCommandError>;

impl Driver {
    pub async fn exec_controller_command<C>(
        &mut self,
        command: C,
        options: Option<ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Option<Command>>
    where
        C: CommandRequest + Clone + 'static,
        SerialFrame: From<C>,
    {
        let result = self.execute_serial_api_command(command).await;
        // TODO: Handle retrying etc.
        match result {
            Ok(SerialApiMachineResult::Success(command)) => Ok(command),
            Ok(result) => Err(result.into()),
            _ => Err(ControllerCommandError::Unexpected(
                "unexpected error".to_string(),
            )),
        }
    }
}

#[derive(Error, Debug)]
pub enum ControllerCommandError {
    #[error("ACK timeout")]
    ACKTimeout,
    #[error("Failed to execute due to repeated CAN")]
    CAN,
    #[error("Failed to execute due to repeated NAK")]
    NAK,
    #[error("Response timeout")]
    ResponseTimeout,
    #[error("The response indicated an error")]
    ResponseNOK(Command),
    #[error("Callback timeout")]
    CallbackTimeout,
    #[error("The callback indicated an error")]
    CallbackNOK(Command),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<SerialApiMachineResult> for ControllerCommandError {
    fn from(result: SerialApiMachineResult) -> Self {
        match result {
            SerialApiMachineResult::ACKTimeout => ControllerCommandError::ACKTimeout,
            SerialApiMachineResult::CAN => ControllerCommandError::CAN,
            SerialApiMachineResult::NAK => ControllerCommandError::NAK,
            SerialApiMachineResult::ResponseTimeout => ControllerCommandError::ResponseTimeout,
            SerialApiMachineResult::ResponseNOK(command) => {
                ControllerCommandError::ResponseNOK(command)
            }
            SerialApiMachineResult::CallbackTimeout => ControllerCommandError::CallbackTimeout,
            SerialApiMachineResult::CallbackNOK(command) => {
                ControllerCommandError::CallbackNOK(command)
            }
            _ => panic!("Serial API machine result is not an error: {:?}", result),
        }
    }
}

macro_rules! expect_controller_command_result {
    ($actual:ident, $expected:ident) => {
        match $actual {
            Ok(Some(Command::$expected(result))) => result,
            Ok(_) => {
                return Err(ControllerCommandError::Unexpected(
                    concat!("expected ", stringify!($expected)).to_string(),
                ))
            }
            Err(e) => return Err(e),
        }
    };
}
pub(crate) use expect_controller_command_result;
