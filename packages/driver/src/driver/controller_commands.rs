use crate::driver::SerialTaskCommand;
use crate::driver::UseNodeIDType;
use crate::exec_background_task;
use crate::Driver;
use crate::SerialApiMachineResult;

use derive_builder::Builder;
use thiserror::Error;
use zwave_core::definitions::NodeId;
use zwave_core::definitions::NodeIdType;
use zwave_core::definitions::Powerlevel;
use zwave_core::definitions::RfRegion;
use zwave_serial::command::GetControllerCapabilitiesRequest;
use zwave_serial::command::GetControllerCapabilitiesResponse;
use zwave_serial::command::GetControllerIdRequest;
use zwave_serial::command::GetControllerIdResponse;
use zwave_serial::command::GetControllerVersionRequest;
use zwave_serial::command::GetControllerVersionResponse;
use zwave_serial::command::GetProtocolVersionRequest;
use zwave_serial::command::GetProtocolVersionResponse;
use zwave_serial::command::GetSerialApiCapabilitiesRequest;
use zwave_serial::command::GetSerialApiCapabilitiesResponse;
use zwave_serial::command::GetSerialApiInitDataRequest;
use zwave_serial::command::GetSerialApiInitDataResponse;
use zwave_serial::command::GetSucNodeIdRequest;
use zwave_serial::command::SerialApiSetupCommand;
use zwave_serial::command::SerialApiSetupRequest;
use zwave_serial::command::SerialApiSetupResponsePayload;
use zwave_serial::{
    command::{Command, CommandRequest},
    frame::SerialFrame,
};

// FIXME: Having a wrapper for this with the correct command options set would be nicer API-wise

impl<P> Driver<P>
where
    P: DriverPhase,
{
    pub async fn get_serial_api_capabilities(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetSerialApiCapabilitiesResponse> {
        println!("Querying Serial API capabilities...");
        let response = self
            .exec_controller_command(GetSerialApiCapabilitiesRequest::default(), options)
            .await;

        let capabilities =
            expect_controller_command_result!(response, GetSerialApiCapabilitiesResponse);

        // TODO: Log response

        Ok(capabilities)
    }

    pub async fn get_serial_api_init_data(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetSerialApiInitDataResponse> {
        println!("Querying Serial API init data...");
        let response = self
            .exec_controller_command(GetSerialApiInitDataRequest::default(), options)
            .await;

        let init_data = expect_controller_command_result!(response, GetSerialApiInitDataResponse);

        // TODO: Log response

        Ok(init_data)
    }

    pub async fn get_controller_capabilities(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerCapabilitiesResponse> {
        println!("Querying controller capabilities...");
        let response = self
            .exec_controller_command(GetControllerCapabilitiesRequest::default(), options)
            .await;

        let capabilities =
            expect_controller_command_result!(response, GetControllerCapabilitiesResponse);

        // TODO: Log response

        Ok(capabilities)
    }

    pub async fn get_controller_version(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerVersionResponse> {
        println!("Querying version info...");
        let response = self
            .exec_controller_command(GetControllerVersionRequest::default(), options)
            .await;

        let version_info =
            expect_controller_command_result!(response, GetControllerVersionResponse);

        // TODO: Log response

        Ok(version_info)
    }

    pub async fn get_controller_id(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerIdResponse> {
        println!("Querying controller ID...");
        let response = self
            .exec_controller_command(GetControllerIdRequest::default(), options)
            .await;

        let ids = expect_controller_command_result!(response, GetControllerIdResponse);

        // TODO: Log response

        Ok(ids)
    }

    pub async fn get_protocol_version(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetProtocolVersionResponse> {
        println!("Querying protocol version...");
        let response = self
            .exec_controller_command(GetProtocolVersionRequest::default(), options)
            .await;

        let protocol_version =
            expect_controller_command_result!(response, GetProtocolVersionResponse);

        // TODO: Log response

        Ok(protocol_version)
    }

    pub async fn get_suc_node_id(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Option<NodeId>> {
        println!("Querying SUC node ID...");
        let response = self
            .exec_controller_command(GetSucNodeIdRequest::default(), options)
            .await;

        let suc_node_id =
            expect_controller_command_result!(response, GetSucNodeIdResponse).suc_node_id;

        // TODO: Log response

        Ok(suc_node_id)
    }

    pub async fn get_supported_serial_api_setup_commands(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Vec<SerialApiSetupCommand>> {
        println!("Querying supported Serial API setup commands...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_supported_commands(), options)
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
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<bool> {
        println!("Switching serial API to {} node IDs...", node_id_type);
        let response = self
            .exec_controller_command(
                SerialApiSetupRequest::set_node_id_type(node_id_type),
                options,
            )
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
            self.storage.node_id_type = node_id_type;
            exec_background_task!(
                &self.tasks.serial_cmd,
                SerialTaskCommand::UseNodeIDType,
                node_id_type
            )?;
        }

        Ok(success)
    }

    pub async fn get_rf_region(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<RfRegion> {
        println!("Querying configured RF region...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_rf_region(), options)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let rf_region = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::GetRFRegion { region } => region
        )?;

        println!("The controller is using RF region {}", rf_region);

        Ok(rf_region)
    }

    pub async fn get_powerlevel(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Powerlevel> {
        println!("Querying configured powerlevel...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_powerlevel(), options)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let powerlevel = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::GetPowerlevel { powerlevel } => powerlevel
        )?;

        println!("The controller is using powerlevel {}", powerlevel);

        Ok(powerlevel)
    }
}

impl<P> Driver<P>
where
    P: DriverPhase,
{
    pub async fn exec_controller_command<C>(
        &mut self,
        command: C,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ExecControllerCommandResult<Option<Command>>
    where
        C: CommandRequest + Clone + 'static,
        SerialFrame: From<C>,
    {
        let options = match options {
            Some(options) => options.clone(),
            None => Default::default(),
        };

        let supported = self.phase.supports_function(command.function_type());
        if options.enforce_support && !supported {
            return Err(ExecControllerCommandError::Unsupported(format!(
                "{:?}",
                command.function_type()
            )));
        }

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

#[derive(Builder, Default, Clone)]
#[builder(setter(into, strip_option), default)]
pub struct ExecControllerCommandOptions {
    /// If executing the command should fail when it is not supported by the controller.
    /// Setting this to `false` is is useful if the capabilities haven't been determined yet. Default: `true`
    #[builder(default = "true")]
    enforce_support: bool,
}

impl ExecControllerCommandOptions {
    pub fn builder() -> ExecControllerCommandOptionsBuilder {
        ExecControllerCommandOptionsBuilder::default()
    }
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

use super::DriverPhase;
