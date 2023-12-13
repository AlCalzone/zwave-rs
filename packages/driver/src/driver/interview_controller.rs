use crate::{driver::ControllerCommandError, ControllerCommandResult, Driver};
use crate::{expect_controller_command_result, Controller};

use zwave_core::definitions::{
    parse_libary_version, DeviceFingerprint, FunctionType, NodeIdType,
};
use zwave_serial::command::{
    Command, GetControllerIdRequest, SerialApiSetupCommand,
};

impl Driver {
    pub(crate) async fn interview_controller(&mut self) -> ControllerCommandResult<Controller> {
        // TODO: Log results
        let capabilities = self.get_serial_api_capabilities().await?;
        let version_info = self.get_controller_version().await?;

        // GetProtocolVersion includes the patch version, GetControllerVersion does not.
        // We prefer having this information, so query it if supported.
        let protocol_version = if capabilities
            .supported_function_types
            .contains(&FunctionType::GetProtocolVersion)
        {
            let protocol_version = self.get_protocol_version().await?;

            // TODO: Log build number and hash

            protocol_version.version
        } else {
            parse_libary_version(&version_info.library_version).map_err(|e| {
                ControllerCommandError::Unexpected(format!("Failed to parse library version: {e}"))
            })?
        };

        let supported_serial_api_setup_commands = if capabilities
            .supported_function_types
            .contains(&FunctionType::SerialApiSetup)
        {
            self.get_supported_serial_api_setup_commands().await?
        } else {
            vec![]
        };

        // Switch to 16 bit node IDs if supported. We need to do this here, as a controller may still be
        // in 16 bit mode when Z-Wave starts up. This would lead to an invalid node ID being reported.
        if supported_serial_api_setup_commands.contains(&SerialApiSetupCommand::SetNodeIDType) {
            let _ = self.set_node_id_type(NodeIdType::NodeId16Bit).await?;
        }

        let response = self
            .exec_controller_command(GetControllerIdRequest::default(), None)
            .await;
        let ids = expect_controller_command_result!(response, GetControllerIdResponse);

        Ok(Controller::new(
            ids.home_id,
            ids.own_node_id,
            DeviceFingerprint::new(
                capabilities.manufacturer_id,
                capabilities.product_type,
                capabilities.product_id,
                capabilities.firmware_version,
            ),
            version_info.library_type,
            protocol_version,
            capabilities.supported_function_types,
            supported_serial_api_setup_commands,
        ))
    }
}
