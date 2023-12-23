use std::collections::BTreeMap;

use custom_debug_derive::Debug;
use typed_builder::TypedBuilder;
use zwave_core::definitions::{
    ControllerRole, DeviceFingerprint, FunctionType, NodeId, NodeType, Powerlevel, RfRegion,
    Version, ZWaveApiVersion, ZWaveLibraryType,
};
use zwave_serial::command::SerialApiSetupCommand;

use crate::Node;

#[derive(Debug, TypedBuilder)]
pub struct Controller {
    #[debug(format = "0x{:08x}")]
    home_id: u32,
    own_node_id: NodeId,
    suc_node_id: Option<NodeId>,
    nodes: BTreeMap<NodeId, Node>,

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

    #[builder(setter(skip), default)]
    rf_region: Option<RfRegion>,
    #[builder(setter(skip), default)]
    powerlevel: Option<Powerlevel>,
}

impl Controller {
    /// Checks whether a given Z-Wave function type is supported by the controller.
    pub fn supports_function(&self, function_type: FunctionType) -> bool {
        self.supported_function_types.contains(&function_type)
    }

    /// Checks whether a given Z-Wave Serial API setup command is supported by the controller.
    pub fn supports_serial_api_setup_command(&self, command: SerialApiSetupCommand) -> bool {
        self.supported_serial_api_setup_commands.contains(&command)
    }

    pub fn home_id(&self) -> u32 {
        self.home_id
    }

    pub fn own_node_id(&self) -> NodeId {
        self.own_node_id
    }

    pub fn suc_node_id(&self) -> Option<NodeId> {
        self.suc_node_id
    }

    pub(crate) fn set_suc_node_id(&mut self, suc_node_id: Option<NodeId>) {
        self.suc_node_id = suc_node_id;
    }

    pub fn is_suc(&self) -> bool {
        self.is_suc
    }

    pub(crate) fn set_is_suc(&mut self, is_suc: bool) {
        self.is_suc = is_suc;
    }

    pub fn is_sis(&self) -> bool {
        self.is_sis
    }

    pub(crate) fn set_is_sis(&mut self, is_sis: bool) {
        self.is_sis = is_sis;
    }

    pub fn sis_present(&self) -> bool {
        self.sis_present
    }

    pub(crate) fn set_sis_present(&mut self, sis_present: bool) {
        self.sis_present = sis_present;
    }

    pub fn role(&self) -> ControllerRole {
        self.role
    }

    pub(crate) fn set_role(&mut self, role: ControllerRole) {
        self.role = role;
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

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().map(|(_, node)| node)
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.iter_mut().map(|(_, node)| node)
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<&Node> {
        self.nodes.get(&node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&node_id)
    }

}
