use crate::driver::SerialTaskCommand;
use crate::driver::UseNodeIDType;
use crate::exec_background_task;
use crate::Driver;
use crate::SerialApiMachineResult;

use thiserror::Error;
use zwave_core::definitions::NodeIdType;
use zwave_serial::command::GetControllerVersionRequest;
use zwave_serial::command::GetControllerVersionResponse;
use zwave_serial::command::GetProtocolVersionRequest;
use zwave_serial::command::GetProtocolVersionResponse;
use zwave_serial::command::GetSerialApiCapabilitiesRequest;
use zwave_serial::command::GetSerialApiCapabilitiesResponse;
use zwave_serial::command::SerialApiSetupCommand;
use zwave_serial::command::SerialApiSetupRequest;
use zwave_serial::command::SerialApiSetupResponsePayload;
use zwave_serial::{
    command::{Command, CommandRequest},
    frame::SerialFrame,
};

impl Driver {
    pub async fn get_serial_api_capabilities(
        &mut self,
    ) -> ControllerCommandResult<GetSerialApiCapabilitiesResponse> {
        println!("Querying Serial API capabilities...");
        let response = self
            .exec_controller_command(GetSerialApiCapabilitiesRequest::default(), None)
            .await;

        let capabilities =
            expect_controller_command_result!(response, GetSerialApiCapabilitiesResponse);

        Ok(capabilities)
    }

    pub async fn get_controller_version(
        &mut self,
    ) -> ControllerCommandResult<GetControllerVersionResponse> {
        println!("Querying version info...");
        let response = self
            .exec_controller_command(GetControllerVersionRequest::default(), None)
            .await;

        let version_info =
            expect_controller_command_result!(response, GetControllerVersionResponse);

        Ok(version_info)
    }

    pub async fn get_protocol_version(
        &mut self,
    ) -> ControllerCommandResult<GetProtocolVersionResponse> {
        println!("Querying protocol version...");
        let response = self
            .exec_controller_command(GetProtocolVersionRequest::default(), None)
            .await;

        let protocol_version =
            expect_controller_command_result!(response, GetProtocolVersionResponse);

        Ok(protocol_version)
    }

    pub async fn get_supported_serial_api_setup_commands(
        &mut self,
    ) -> ControllerCommandResult<Vec<SerialApiSetupCommand>> {
        println!("Querying supported Serial API setup commands...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_supported_commands(), None)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        // TODO: Log supported commands
        expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::GetSupportedCommands { commands } => Ok(commands)
        )?
    }

    pub async fn set_node_id_type(
        &mut self,
        node_id_type: NodeIdType,
    ) -> ControllerCommandResult<bool> {
        println!("Switching serial API to {} node IDs...", node_id_type);
        let response = self
            .exec_controller_command(SerialApiSetupRequest::set_node_id_type(node_id_type), None)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let success = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::SetNodeIDType { success } => success
        )?;

        println!(
            "Switching serial API to {} node IDs {}",
            node_id_type,
            if success { "succeeded" } else { "failed" }
        );

        if success {
            self.state.node_id_type = node_id_type;
            exec_background_task!(
                &self.serial_cmd,
                SerialTaskCommand::UseNodeIDType,
                node_id_type
            )?;
        }

        Ok(success)
    }

    pub async fn exec_controller_command<C>(
        &mut self,
        command: C,
        options: Option<ExecControllerCommandOptions>,
    ) -> ExecControllerCommandResult<Option<Command>>
    where
        C: CommandRequest + Clone + 'static,
        SerialFrame: From<C>,
    {
        let result = self.execute_serial_api_command(command).await;
        // TODO: Handle retrying etc.
        match result {
            Ok(SerialApiMachineResult::Success(command)) => Ok(command),
            Ok(result) => Err(result.into()),
            _ => Err(ExecControllerCommandError::Unexpected(
                "unexpected error".to_string(),
            )),
        }
    }
}

pub struct ExecControllerCommandOptions {}

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
                return Err(ControllerCommandError::Unexpected(
                    concat!("expected ", stringify!($expected)).to_string(),
                ))
            }
            Err(e) => return Err(e.into()),
        }
    };
}
pub(crate) use expect_controller_command_result;

macro_rules! expect_serial_api_setup_result {
    ($actual:expr, $expected:pat => $result:expr) => {
        match $actual {
            $expected => Ok($result),
            SerialApiSetupResponsePayload::Unsupported(cmd) => Err(
                ControllerCommandError::Unsupported(format!("SerialApiSetup::{:?}", cmd)),
            ),
            _ => Err(ControllerCommandError::Unexpected(
                "Unexpected SerialApiSetup response payload".to_string(),
            )),
        }
    };
}
pub(crate) use expect_serial_api_setup_result;
