use crate::driver::SerialTaskCommand;
use crate::driver::UseNodeIDType;
use crate::exec_background_task;
use crate::Driver;
use crate::Ready;
use crate::SerialApiMachineResult;

use thiserror::Error;
use typed_builder::TypedBuilder;
use zwave_core::prelude::*;
use zwave_serial::command::{
    Command, CommandBase, CommandRequest, GetControllerCapabilitiesRequest,
    GetControllerCapabilitiesResponse, GetControllerIdRequest, GetControllerIdResponse,
    GetControllerVersionRequest, GetControllerVersionResponse, GetNodeProtocolInfoRequest,
    GetNodeProtocolInfoResponse, GetProtocolVersionRequest, GetProtocolVersionResponse,
    GetSerialApiCapabilitiesRequest, GetSerialApiCapabilitiesResponse, GetSerialApiInitDataRequest,
    GetSerialApiInitDataResponse, GetSucNodeIdRequest, SerialApiSetupCommand,
    SerialApiSetupRequest, SerialApiSetupResponsePayload, SetSucNodeIdRequest,
};
use zwave_serial::frame::SerialFrame;

// FIXME: Having a wrapper for this with the correct command options set would be nicer API-wise

// Define the commands that can be executed in any phase
impl<S> Driver<S>
where
    S: DriverState,
{
    pub async fn get_serial_api_capabilities(
        &self,
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
        &self,
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
        &self,
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
        &self,
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
        &self,
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
        &self,
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
        &self,
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
        &self,
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
}

// Define the commands that require the driver to be ready
impl Driver<Ready> {
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

        self.state.controller.set_rf_region(Some(rf_region));

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

        self.state.controller.set_powerlevel(Some(powerlevel));

        println!("The controller is using powerlevel {}", powerlevel);

        Ok(powerlevel)
    }

    pub async fn set_tx_status_report(
        &self,
        enabled: bool,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<bool> {
        println!(
            "{} TX status reports...",
            if enabled { "Enabling" } else { "Disabling" }
        );
        let response = self
            .exec_controller_command(
                SerialApiSetupRequest::set_tx_status_report(enabled),
                options,
            )
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let success = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::SetTxStatusReport { success } => success
        )?;

        println!(
            "{} TX status reports {}",
            if enabled { "Enabling" } else { "Disabling" },
            if success { "succeeded" } else { "failed" }
        );

        Ok(success)
    }

    pub async fn set_suc_node_id(
        &mut self,
        own_node_id: NodeId,
        node_id: NodeId,
        enable_suc: bool,
        enable_sis: bool,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<bool> {
        let cmd = SetSucNodeIdRequest::builder()
            .own_node_id(own_node_id)
            .suc_node_id(node_id)
            .enable_suc(enable_suc)
            .enable_sis(enable_sis)
            .build();

        let response = self.exec_controller_command(cmd, options).await;
        let success = match response {
            Ok(Some(Command::SetSucNodeIdResponse(result))) => result.is_ok(),
            Ok(Some(Command::SetSucNodeIdCallback(result))) => result.is_ok(),
            Ok(_) => {
                return Err(ControllerCommandError::Unexpected(
                    "expected SetSucNodeIdResponse or SetSucNodeIdCallback".to_string(),
                ))
            }
            Err(e) => return Err(e.into()),
        };

        if success {
            self.state.controller.set_suc_node_id(Some(node_id));
            // FIXME: If we promoted ourselves also set the is_suc/is_sis/sis_present flags to true
        }

        Ok(success)
    }

    pub async fn get_node_protocol_info(
        &self,
        node_id: &NodeId,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<NodeInformationProtocolData> {
        let cmd = GetNodeProtocolInfoRequest { node_id: *node_id };
        let response = self.exec_controller_command(cmd, options).await;
        let response = expect_controller_command_result!(response, GetNodeProtocolInfoResponse);

        Ok(response.protocol_info)
    }
}

impl<S> Driver<S>
where
    S: DriverState,
{
    pub async fn exec_controller_command<C>(
        &self,
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

        let supported = self.state.supports_function(command.function_type());
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

#[derive(TypedBuilder, Default, Clone)]
pub struct ExecControllerCommandOptions {
    /// If executing the command should fail when it is not supported by the controller.
    /// Setting this to `false` is is useful if the capabilities haven't been determined yet. Default: `true`
    #[builder(default = true)]
    enforce_support: bool,
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

use super::DriverState;
