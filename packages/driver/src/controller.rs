use crate::{driver_api::DriverApi, Ready};
use std::sync::{atomic::Ordering, Arc};
use zwave_core::{definitions::*, submodule};
use zwave_serial::command::SerialApiSetupCommand;

submodule!(storage);

macro_rules! read {
    ($self:ident, $field:ident) => {
        $self.storage.$field
    };
}

macro_rules! read_locked {
    ($self:ident, $field:ident) => {
        *$self.storage.$field.read().unwrap()
    };
}

macro_rules! write_locked {
    ($self:ident, $field:ident, $value:expr) => {
        *$self.storage.$field.write().unwrap() = $value;
    };
}

macro_rules! read_atomic {
    ($self:ident, $field:ident) => {
        read!($self, $field).load(Ordering::Relaxed)
    };
}

macro_rules! write_atomic {
    ($self:ident, $field:ident, $value:expr) => {
        read!($self, $field).store($value, Ordering::Relaxed);
    };
}

// API access for the controller instance
impl DriverApi<Ready> {
    pub fn controller(&self) -> Controller {
        Controller::new(self.state.controller.clone())
    }
}

// #[derive(Debug)]
pub struct Controller {
    storage: Arc<ControllerStorage>,
}

impl Controller {
    pub fn new(storage: Arc<ControllerStorage>) -> Self {
        Self { storage }
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

    pub fn own_node_id(&self) -> NodeId {
        read!(self, own_node_id)
    }

    pub fn suc_node_id(&self) -> Option<NodeId> {
        read_locked!(self, suc_node_id)
    }

    pub(crate) fn set_suc_node_id(&mut self, suc_node_id: Option<NodeId>) {
        write_locked!(self, suc_node_id, suc_node_id);
    }

    pub fn is_suc(&self) -> bool {
        read_atomic!(self, is_suc)
    }

    pub(crate) fn set_is_suc(&mut self, is_suc: bool) {
        write_atomic!(self, is_suc, is_suc);
    }

    pub fn is_sis(&self) -> bool {
        read_atomic!(self, is_sis)
    }

    pub(crate) fn set_is_sis(&mut self, is_sis: bool) {
        write_atomic!(self, is_sis, is_sis);
    }

    pub fn sis_present(&self) -> bool {
        read_atomic!(self, sis_present)
    }

    pub(crate) fn set_sis_present(&mut self, sis_present: bool) {
        write_atomic!(self, sis_present, sis_present);
    }

    pub fn role(&self) -> ControllerRole {
        read_locked!(self, role)
    }

    pub(crate) fn set_role(&mut self, role: ControllerRole) {
        write_locked!(self, role, role);
    }

    pub fn rf_region(&self) -> Option<RfRegion> {
        read_locked!(self, rf_region)
    }

    pub(crate) fn set_rf_region(&mut self, region: Option<RfRegion>) {
        write_locked!(self, rf_region, region);
    }

    pub fn powerlevel(&self) -> Option<Powerlevel> {
        read_locked!(self, powerlevel)
    }

    pub(crate) fn set_powerlevel(&mut self, powerlevel: Option<Powerlevel>) {
        write_locked!(self, powerlevel, powerlevel);
    }
}
