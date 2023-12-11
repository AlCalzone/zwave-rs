use crate::expect_controller_command_result;
use crate::{driver::ControllerCommandError, ControllerCommandResult, Driver};
use custom_debug_derive::Debug;
use zwave_core::definitions::{FunctionType, Version, ZWaveLibraryType};
use zwave_serial::command::{
    Command, GetControllerIdRequest, GetControllerVersionRequest, GetSerialApiCapabilitiesRequest,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ControllerInfo {
    #[debug(format = "0x{:04x}")]
    manufacturer_id: u16,
    #[debug(format = "0x{:04x}")]
    product_type: u16,
    #[debug(format = "0x{:04x}")]
    product_id: u16,
    firmware_version: Version,
    supported_function_types: Vec<FunctionType>,

    library_type: ZWaveLibraryType,
    library_version: String,

    #[debug(format = "0x{:08x}")]
    home_id: u32,
    #[debug(format = "0x{:04x}")]
    own_node_id: u16,
}

impl Driver {
    pub(crate) async fn identify_controller(&mut self) -> ControllerCommandResult<ControllerInfo> {
        println!("Querying Serial API capabilities...");
        let response = self
            .exec_controller_command(GetSerialApiCapabilitiesRequest::default(), None)
            .await;

        let capabilities =
            expect_controller_command_result!(response, GetSerialApiCapabilitiesResponse);

        // TODO: Log result

        println!("Querying version info...");
        let response = self
            .exec_controller_command(GetControllerVersionRequest::default(), None)
            .await;

        let version_info =
            expect_controller_command_result!(response, GetControllerVersionResponse);

        // TODO: If supported, use GetProtocolVersion

        // TODO: If supported, switch to 16-bit node IDs

        let response = self
            .exec_controller_command(GetControllerIdRequest::default(), None)
            .await;
        let ids = expect_controller_command_result!(response, GetControllerIdResponse);

        Ok(ControllerInfo {
            manufacturer_id: capabilities.manufacturer_id,
            product_type: capabilities.product_type,
            product_id: capabilities.product_id,
            firmware_version: capabilities.firmware_version,
            supported_function_types: capabilities.supported_function_types,
            library_type: version_info.library_type,
            library_version: version_info.library_version,
            home_id: ids.home_id,
            own_node_id: ids.own_node_id,
        })
    }
}
