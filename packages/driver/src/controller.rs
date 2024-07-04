use crate::{driver_api::DriverApi, Ready};
use std::sync::{atomic::Ordering, Arc};
use zwave_core::{definitions::*, submodule};
use zwave_serial::command::SerialApiSetupCommand;

submodule!(storage);

macro_rules! read {
    ($self:ident, $field:ident) => {
        $self
            .driver
            .storage
            .controller()
            .as_ref()
            .expect("attempted to read controller storage before initialization")
            .$field
    };
}

macro_rules! write {
    ($self:ident, $field:ident, $value:expr) => {
        $self
            .driver
            .storage
            .controller_mut()
            .as_mut()
            .expect("attempted to read controller storage before initialization")
            .$field = $value;
    };
}

// API access for the controller instance
impl DriverApi {
    // FIXME: Assert that the driver is in the Ready state
    pub fn controller(&self) -> Controller {
        Controller::new(self)
    }

    pub fn own_node_id(&self) -> NodeId {
        self.storage
            .controller()
            .as_ref()
            .map(|c| c.own_node_id)
            .unwrap_or(NodeId::unspecified())
    }
}

// #[derive(Debug)]
pub struct Controller<'a> {
    driver: &'a DriverApi,
}

impl<'a> Controller<'a> {
    pub fn new(driver: &'a DriverApi) -> Self {
        Self { driver }
    }

    /// Checks whether a given Z-Wave function type is supported by the controller.
    pub fn supports_function(&self, function_type: FunctionType) -> bool {
        read!(self, supported_function_types).contains(&function_type)
    }

    /// Checks whether a given Z-Wave Serial API setup command is supported by the controller.
    pub fn supports_serial_api_setup_command(&self, command: SerialApiSetupCommand) -> bool {
        read!(self, supported_serial_api_setup_commands).contains(&command)
    }

    pub fn home_id(&self) -> u32 {
        read!(self, home_id)
    }

    pub fn suc_node_id(&self) -> Option<NodeId> {
        read!(self, suc_node_id)
    }

    pub(crate) fn set_suc_node_id(&mut self, suc_node_id: Option<NodeId>) {
        write!(self, suc_node_id, suc_node_id);
    }

    pub fn is_suc(&self) -> bool {
        read!(self, is_suc)
    }

    pub(crate) fn set_is_suc(&mut self, is_suc: bool) {
        write!(self, is_suc, is_suc);
    }

    pub fn is_sis(&self) -> bool {
        read!(self, is_sis)
    }

    pub(crate) fn set_is_sis(&mut self, is_sis: bool) {
        write!(self, is_sis, is_sis);
    }

    pub fn sis_present(&self) -> bool {
        read!(self, sis_present)
    }

    pub(crate) fn set_sis_present(&mut self, sis_present: bool) {
        write!(self, sis_present, sis_present);
    }

    pub fn role(&self) -> ControllerRole {
        read!(self, role)
    }

    pub(crate) fn set_role(&mut self, role: ControllerRole) {
        write!(self, role, role);
    }

    pub fn rf_region(&self) -> Option<RfRegion> {
        read!(self, rf_region)
    }

    pub(crate) fn set_rf_region(&mut self, region: Option<RfRegion>) {
        write!(self, rf_region, region);
    }

    pub fn powerlevel(&self) -> Option<Powerlevel> {
        read!(self, powerlevel)
    }

    pub(crate) fn set_powerlevel(&mut self, powerlevel: Option<Powerlevel>) {
        write!(self, powerlevel, powerlevel);
    }
}
