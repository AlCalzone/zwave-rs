use custom_debug_derive::Debug;
use std::sync::{RwLock, atomic::AtomicBool};
use zwave_core::prelude::*;
use typed_builder::TypedBuilder;
use zwave_serial::command::SerialApiSetupCommand;

#[derive(Debug, TypedBuilder)]
/// Internal storage for the controller instance. Since this is meant be used from both library and external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct ControllerStorage {
    #[debug(format = "0x{:08x}")]
    pub(crate) home_id: u32,
    pub(crate) own_node_id: NodeId,
    #[builder(setter(into))]
    pub(crate) suc_node_id: RwLock<Option<NodeId>>,

    pub(crate) fingerprint: DeviceFingerprint,

    pub(crate) library_type: ZWaveLibraryType,
    pub(crate) api_version: ZWaveApiVersion,
    pub(crate) protocol_version: Version,
    pub(crate) sdk_version: Version,

    pub(crate) node_type: NodeType,
    #[builder(setter(into))]
    pub(crate) role: RwLock<ControllerRole>,
    pub(crate) started_this_network: bool,
    #[builder(setter(into))]
    pub(crate) sis_present: AtomicBool,
    #[builder(setter(into))]
    pub(crate) is_sis: AtomicBool,
    #[builder(setter(into))]
    pub(crate) is_suc: AtomicBool,

    pub(crate) supported_function_types: Vec<FunctionType>,
    pub(crate) supported_serial_api_setup_commands: Vec<SerialApiSetupCommand>,
    pub(crate) supports_timers: bool,

    #[builder(setter(skip), default)]
    pub(crate) rf_region: RwLock<Option<RfRegion>>,
    #[builder(setter(skip), default)]
    pub(crate) powerlevel: RwLock<Option<Powerlevel>>,
}
