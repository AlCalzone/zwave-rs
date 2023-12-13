use crate::{driver::ControllerCommandError, ControllerCommandResult, Driver};
use crate::{expect_controller_command_result, Controller};

use zwave_core::definitions::{
    parse_libary_version, protocol_version_to_sdk_version, DeviceFingerprint, FunctionType,
    NodeIdType,
};
use zwave_serial::command::{Command, GetControllerIdRequest, SerialApiSetupCommand};

impl Driver {
    pub(crate) async fn interview_controller(&mut self) -> ControllerCommandResult<Controller> {
        // TODO: Log results
        let api_capabilities = self.get_serial_api_capabilities().await?;
        let init_data = self.get_serial_api_init_data().await?;
        let version_info = self.get_controller_version().await?;
        let capabilities = self.get_controller_capabilities().await?;

        // GetProtocolVersion includes the patch version, GetControllerVersion does not.
        // We prefer having this information, so query it if supported.
        let protocol_version = if api_capabilities
            .supported_function_types
            .contains(&FunctionType::GetProtocolVersion)
        {
            self.get_protocol_version().await?.version
        } else {
            parse_libary_version(&version_info.library_version).map_err(|e| {
                ControllerCommandError::Unexpected(format!("Failed to parse library version: {e}"))
            })?
        };

        let supported_serial_api_setup_commands = if api_capabilities
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

        // Afterwards, execute the commands that parse node IDs
        let ids = self.get_controller_id().await?;
        let suc_node_id = self.get_suc_node_id().await?;

        let controller = Controller::builder()
            .home_id(ids.home_id)
            .own_node_id(ids.own_node_id)
            .suc_node_id(suc_node_id)
            .node_ids(init_data.node_ids)
            .fingerprint(DeviceFingerprint::new(
                api_capabilities.manufacturer_id,
                api_capabilities.product_type,
                api_capabilities.product_id,
                api_capabilities.firmware_version,
            ))
            .library_type(version_info.library_type)
            .api_version(init_data.api_version)
            .protocol_version(protocol_version)
            .sdk_version(protocol_version_to_sdk_version(&protocol_version))
            .node_type(init_data.node_type)
            .role(capabilities.role)
            .started_this_network(capabilities.started_this_network)
            .sis_present(capabilities.sis_present)
            .is_sis(init_data.is_sis)
            .is_suc(capabilities.is_suc)
            .supported_function_types(api_capabilities.supported_function_types)
            .supported_serial_api_setup_commands(supported_serial_api_setup_commands)
            .supports_timers(init_data.supports_timers)
            .build()
            .unwrap();

        Ok(controller)
    }
}
