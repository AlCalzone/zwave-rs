use std::collections::BTreeMap;
use zwave_core::definitions::{FunctionType, NodeId};
use crate::{ControllerStorage, NodeStorage};

/// The driver can be in one of multiple states, each of which has a different set of capabilities.
pub trait DriverState {
    /// Whether the driver supports executing the given function type in this phase
    #[allow(unused_variables)]
    fn supports_function(&self, function_type: FunctionType) -> bool {
        // By default: Don't know, don't care
        false
    }
}

/// The driver isn't fully initialized yet
pub struct Init;
impl DriverState for Init {}

/// The driver is ready to use normally
#[derive(Debug)]
pub struct Ready {
    pub(crate) controller: ControllerStorage,
    pub(crate) nodes: BTreeMap<NodeId, NodeStorage>,
}

impl DriverState for Ready {
    fn supports_function(&self, function_type: FunctionType) -> bool {
        self.controller
            .supported_function_types
            .contains(&function_type)
    }
}
