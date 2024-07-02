use std::{collections::BTreeMap, sync::Arc};
use zwave_core::definitions::{FunctionType, NodeId};
use crate::{ControllerStorage, NodeStorage};

/// The driver can be in one of multiple states, each of which has a different set of capabilities.
pub trait DriverState: Clone {
    /// Whether the driver supports executing the given function type in this phase
    #[allow(unused_variables)]
    fn supports_function(&self, function_type: FunctionType) -> bool {
        // By default: Don't know, don't care
        false
    }
}

/// The driver isn't fully initialized yet
#[derive(Debug, Clone)]
pub struct Init;
impl DriverState for Init {}

/// The driver is ready to use normally
#[derive(Debug, Clone)]
pub struct Ready {
    pub(crate) controller: Arc<ControllerStorage>,
    pub(crate) nodes: Arc<BTreeMap<NodeId, NodeStorage>>,
}

impl DriverState for Ready {
    fn supports_function(&self, function_type: FunctionType) -> bool {
        self.controller
            .supported_function_types
            .contains(&function_type)
    }
}
