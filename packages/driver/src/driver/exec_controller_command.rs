use super::Driver;
use thiserror::Error;
use typed_builder::TypedBuilder;
use zwave_serial::command::Command;

impl Driver {
    pub async fn exec_controller_command<C>(
        &self,
        command: C,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ExecControllerCommandResult<Option<Command>>
    where
        C: ExecutableCommand + 'static,
    {
        // FIXME:
        // let options = match options {
        //     Some(options) => options.clone(),
        //     None => Default::default(),
        // };

        // let supported = self.supports_function(command.function_type());
        // if options.enforce_support && !supported {
        //     return Err(ExecControllerCommandError::Unsupported(format!(
        //         "{:?}",
        //         command.function_type()
        //     )));
        // }

        let result = self.serial_api.execute_serial_api_command(command).await;
        // TODO: Handle retrying etc.
        match result {
            Ok(SerialApiMachineResult::Success(command)) => Ok(command),
            Ok(result) => Err(result.into()),
            Err(e) => Err(ExecControllerCommandError::Unexpected(format!(
                "unexpected error in execute_serial_api_command: {:?}",
                e
            ))),
        }
    }
}

#[derive(TypedBuilder, Default, Clone)]
pub struct ExecControllerCommandOptions {
    // /// If executing the command should fail when it is not supported by the controller.
    // /// Setting this to `false` is is useful if the capabilities haven't been determined yet. Default: `true`
    // #[builder(default = true)]
    // enforce_support: bool,
}

/// The low-level result of a controller command execution.
pub type ExecControllerCommandResult<T> = Result<T, ExecControllerCommandError>;

#[derive(Error, Debug)]
/// Defines the possible low-level errors for a controller command execution
pub enum ExecControllerCommandError {
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
    #[error("Command not supported: {0}")]
    Unsupported(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<SerialApiMachineResult> for ExecControllerCommandError {
    fn from(result: SerialApiMachineResult) -> Self {
        match result {
            SerialApiMachineResult::ACKTimeout => ExecControllerCommandError::ACKTimeout,
            SerialApiMachineResult::CAN => ExecControllerCommandError::CAN,
            SerialApiMachineResult::NAK => ExecControllerCommandError::NAK,
            SerialApiMachineResult::ResponseTimeout => ExecControllerCommandError::ResponseTimeout,
            SerialApiMachineResult::ResponseNOK(command) => {
                ExecControllerCommandError::ResponseNOK(command)
            }
            SerialApiMachineResult::CallbackTimeout => ExecControllerCommandError::CallbackTimeout,
            SerialApiMachineResult::CallbackNOK(command) => {
                ExecControllerCommandError::CallbackNOK(command)
            }
            _ => panic!("Serial API machine result is not an error: {:?}", result),
        }
    }
}

/// The high-level result of a controller command execution.
pub type ControllerCommandResult<T> = Result<T, ControllerCommandError>;

#[derive(Error, Debug)]
/// Defines the possible high-level errors for a controller command execution
pub enum ControllerCommandError {
    #[error("Controller communication failure")]
    Failure,
    #[error("Command was unsuccessful")]
    Unsuccessful,
    #[error("Command not supported: {0}")]
    Unsupported(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<ExecControllerCommandError> for ControllerCommandError {
    fn from(value: ExecControllerCommandError) -> Self {
        match value {
            ExecControllerCommandError::ACKTimeout
            | ExecControllerCommandError::CAN
            | ExecControllerCommandError::NAK
            | ExecControllerCommandError::ResponseTimeout
            | ExecControllerCommandError::CallbackTimeout => ControllerCommandError::Failure,
            ExecControllerCommandError::ResponseNOK(_)
            | ExecControllerCommandError::CallbackNOK(_) => ControllerCommandError::Unsuccessful,
            ExecControllerCommandError::Unsupported(s) => ControllerCommandError::Unsupported(s),
            ExecControllerCommandError::Unexpected(s) => ControllerCommandError::Unexpected(s),
        }
    }
}

impl From<crate::error::Error> for ControllerCommandError {
    fn from(value: crate::error::Error) -> Self {
        ControllerCommandError::Unexpected(value.to_string())
    }
}

macro_rules! expect_controller_command_result {
    ($actual:expr, $expected:ident) => {
        match $actual {
            Ok(Some(Command::$expected(result))) => result,
            Ok(_) => {
                return Err($crate::ControllerCommandError::Unexpected(
                    concat!("expected ", stringify!($expected)).to_string(),
                ))
            }
            Err(e) => return Err(e.into()),
        }
    };
}
pub(crate) use expect_controller_command_result;

use crate::{ExecutableCommand, SerialApiMachineResult};
