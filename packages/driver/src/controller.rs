use custom_debug_derive::Debug;
use derive_builder::Builder;
use zwave_core::definitions::{
    ControllerRole, DeviceFingerprint, FunctionType, NodeId, NodeType, Powerlevel, RfRegion,
    Version, ZWaveApiVersion, ZWaveLibraryType,
};
use zwave_serial::command::SerialApiSetupCommand;

#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(pattern = "owned")]
pub struct Controller {
    #[debug(format = "0x{:08x}")]
    home_id: u32,
    own_node_id: NodeId,
    node_ids: Vec<NodeId>,
    suc_node_id: Option<NodeId>,

    fingerprint: DeviceFingerprint,

    library_type: ZWaveLibraryType,
    api_version: ZWaveApiVersion,
    protocol_version: Version,
    sdk_version: Version,

    node_type: NodeType,
    role: ControllerRole,
    started_this_network: bool,
    sis_present: bool,
    is_sis: bool,
    is_suc: bool,

    supported_function_types: Vec<FunctionType>,
    supported_serial_api_setup_commands: Vec<SerialApiSetupCommand>,
    supports_timers: bool,

    #[builder(setter(skip, strip_option))]
    rf_region: Option<RfRegion>,
    #[builder(setter(skip, strip_option))]
    powerlevel: Option<Powerlevel>,
}

impl Controller {
    pub fn builder() -> ControllerBuilder {
        ControllerBuilder::default()
    }

    /// Checks whether a given Z-Wave function type is supported by the controller.
    pub fn supports_function(&self, function_type: FunctionType) -> bool {
        self.supported_function_types.contains(&function_type)
    }

    /// Checks whether a given Z-Wave Serial API setup command is supported by the controller.
    pub fn supports_serial_api_setup_command(&self, command: SerialApiSetupCommand) -> bool {
        self.supported_serial_api_setup_commands.contains(&command)
    }

    pub fn rf_region(&self) -> Option<RfRegion> {
        self.rf_region
    }

    pub(crate) fn set_rf_region(&mut self, region: Option<RfRegion>) {
        self.rf_region = region;
    }

    pub fn powerlevel(&self) -> Option<Powerlevel> {
        self.powerlevel
    }

    pub(crate) fn set_powerlevel(&mut self, powerlevel: Option<Powerlevel>) {
        self.powerlevel = powerlevel;
    }
}
