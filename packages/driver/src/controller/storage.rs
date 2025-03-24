use typed_builder::TypedBuilder;
use zwave_core::prelude::*;
use zwave_serial::command::SerialApiSetupCommand;

#[derive(Debug, TypedBuilder)]
/// Internal storage for the controller instance. Since this is meant be used from both library and external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct ControllerStorage {
    #[builder(setter(into))]
    pub(crate) home_id: Id32,
    pub(crate) own_node_id: NodeId,
    #[builder(setter(into))]
    pub(crate) suc_node_id: Option<NodeId>,

    pub(crate) fingerprint: DeviceFingerprint,

    pub(crate) library_type: ZWaveLibraryType,
    pub(crate) api_version: ZWaveApiVersion,
    pub(crate) protocol_version: Version,
    pub(crate) sdk_version: Version,

    pub(crate) node_type: NodeType,
    #[builder(setter(into))]
    pub(crate) role: ControllerRole,
    pub(crate) started_this_network: bool,
    #[builder(setter(into))]
    pub(crate) sis_present: bool,
    #[builder(setter(into))]
    pub(crate) is_sis: bool,
    #[builder(setter(into))]
    pub(crate) is_suc: bool,

    pub(crate) supported_function_types: Vec<FunctionType>,
    pub(crate) supported_serial_api_setup_commands: Vec<SerialApiSetupCommand>,
    pub(crate) supports_timers: bool,

    #[builder(setter(skip), default)]
    pub(crate) rf_region: Option<RfRegion>,
    #[builder(setter(skip), default)]
    pub(crate) powerlevel: Option<Powerlevel>,
}
