use custom_debug_derive::Debug;
use zwave_core::definitions::{
    protocol_version_to_sdk_version, DeviceFingerprint, FunctionType, Version, ZWaveLibraryType, NodeId,
};
use zwave_serial::command::SerialApiSetupCommand;

#[derive(Debug, Clone, PartialEq)]
pub struct Controller {
    #[debug(format = "0x{:08x}")]
    home_id: u32,
    own_node_id: NodeId,

    fingerprint: DeviceFingerprint,

    supported_function_types: Vec<FunctionType>,
    supported_serial_api_setup_commands: Vec<SerialApiSetupCommand>,

    library_type: ZWaveLibraryType,
    protocol_version: Version,
    sdk_version: Version,
}

impl Controller {
    pub fn new(
        home_id: u32,
        own_node_id: NodeId,
        fingerprint: DeviceFingerprint,
        library_type: ZWaveLibraryType,
        protocol_version: Version,
        supported_function_types: Vec<FunctionType>,
        supported_serial_api_setup_commands: Vec<SerialApiSetupCommand>,
    ) -> Self {
        Self {
            home_id,
            own_node_id,
            fingerprint,
            library_type,
            protocol_version,
            sdk_version: protocol_version_to_sdk_version(&protocol_version),
            supported_function_types,
            supported_serial_api_setup_commands,
        }
    }

    /// Checks whether a given Z-Wave function type is supported by the controller.
    pub fn supports_function(&self, function_type: FunctionType) -> bool {
        self.supported_function_types.contains(&function_type)
    }

    /// Checks whether a given Z-Wave Serial API setup command is supported by the controller.
    pub fn supports_serial_api_setup_command(&self, command: SerialApiSetupCommand) -> bool {
        self.supported_serial_api_setup_commands.contains(&command)
    }
}
