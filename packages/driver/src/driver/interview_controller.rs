use crate::{driver::ControllerCommandError, ControllerCommandResult, Driver};
use crate::{Controller, ExecControllerCommandOptions, Node};

use zwave_core::definitions::*;
use zwave_serial::command::SerialApiSetupCommand;

use super::{Init, Ready};

impl Driver<Init> {
    pub(crate) async fn interview_controller(&self) -> ControllerCommandResult<Ready> {
        // We execute some of these commands before knowing the controller capabilities, so
        // we disable enforcing that the controller supports the commands.
        let command_options = ExecControllerCommandOptions::builder()
            .enforce_support(false)
            .build();
        let command_options = Some(&command_options);

        // TODO: Log results
        let api_capabilities = self.get_serial_api_capabilities(command_options).await?;
        let init_data = self.get_serial_api_init_data(command_options).await?;
        let version_info = self.get_controller_version(command_options).await?;
        let capabilities = self.get_controller_capabilities(command_options).await?;

        // GetProtocolVersion includes the patch version, GetControllerVersion does not.
        // We prefer having this information, so query it if supported.
        let protocol_version = if api_capabilities
            .supported_function_types
            .contains(&FunctionType::GetProtocolVersion)
        {
            self.get_protocol_version(command_options).await?.version
        } else {
            parse_libary_version(&version_info.library_version).map_err(|e| {
                ControllerCommandError::Unexpected(format!("Failed to parse library version: {e}"))
            })?
        };

        let supported_serial_api_setup_commands = if api_capabilities
            .supported_function_types
            .contains(&FunctionType::SerialApiSetup)
        {
            self.get_supported_serial_api_setup_commands(command_options)
                .await?
        } else {
            vec![]
        };

        // Switch to 16 bit node IDs if supported. We need to do this here, as a controller may still be
        // in 16 bit mode when Z-Wave starts up. This would lead to an invalid node ID being reported.
        if supported_serial_api_setup_commands.contains(&SerialApiSetupCommand::SetNodeIDType) {
            let _ = self
                .set_node_id_type(NodeIdType::NodeId16Bit, command_options)
                .await;
        }

        // Afterwards, execute the commands that parse node IDs
        let ids = self.get_controller_id(command_options).await?;
        let suc_node_id = self.get_suc_node_id(command_options).await?;

        let nodes = init_data.node_ids.iter().map(|node_id| Node::new(*node_id));

        let controller = Controller::builder()
            .home_id(ids.home_id)
            .own_node_id(ids.own_node_id)
            .suc_node_id(suc_node_id)
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
            .build();

        Ok(Ready::new(controller, nodes))
    }
}

impl Driver<Ready> {
    pub(crate) async fn configure_controller(&mut self) -> ControllerCommandResult<()> {
        // Get the currently configured RF region and remember it.
        // If it differs from the desired region, change it afterwards.
        if self
            .controller()
            .supports_serial_api_setup_command(SerialApiSetupCommand::GetRFRegion)
        {
            let _region = self.get_rf_region(None).await?;
            // FIXME: set region if desired
        }

        // Get the currently configured powerlevel and remember it.
        // If it differs from the desired powerlevel, change it afterwards.
        if self
            .controller()
            .supports_serial_api_setup_command(SerialApiSetupCommand::GetPowerlevel)
        {
            let _powerlevel = self.get_powerlevel(None).await?;
            // FIXME: set powerlevel if desired
        }

        // Enable TX status reports if supported
        if self
            .controller()
            .supports_serial_api_setup_command(SerialApiSetupCommand::SetTxStatusReport)
        {
            self.set_tx_status_report(true, None).await?;
        }

        // There needs to be a SUC/SIS in the network.
        // If not, we promote ourselves to SUC if all of the following conditions are met:
        // * We are the primary controller
        // * but we are not SUC
        // * there is no SUC and
        // * there is no SIS
        let should_promote = {
            let controller = self.controller();
            controller.role() == ControllerRole::Primary
                && !controller.is_suc()
                && !controller.is_sis()
                && controller.suc_node_id().is_none()
        };

        if should_promote {
            println!("There is no SUC/SIS in the network - promoting ourselves...");
            let own_node_id = self.controller().own_node_id();
            match self
                .set_suc_node_id(own_node_id, own_node_id, true, true, None)
                .await
            {
                Ok(success) => {
                    println!(
                        "Promotion to SUC/SIS {}",
                        if success { "succeeded" } else { "failed" }
                    );
                }
                Err(e) => {
                    println!("Error while promoting to SUC/SIS: {:?}", e);
                }
            }
        } else {
            println!("There is a SUC/SIS in the network - not promoting ourselves");
        }

        Ok(())
    }
}
