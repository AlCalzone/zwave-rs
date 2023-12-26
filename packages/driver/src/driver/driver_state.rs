use std::{collections::BTreeMap, sync::RwLock};

use zwave_core::definitions::{FunctionType, NodeId};

use crate::{Controller, NodeStorage};

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
    pub(crate) controller: RwLock<Controller>,
    pub(crate) nodes: BTreeMap<NodeId, NodeStorage>,
}

impl Ready {
    pub(crate) fn new(controller: Controller, nodes: BTreeMap<NodeId, NodeStorage>) -> Self {
        Self {
            controller: RwLock::new(controller),
            nodes,
        }
    }
}

impl DriverState for Ready {
    fn supports_function(&self, function_type: FunctionType) -> bool {
        let controller = self.controller.read().unwrap();
        controller.supports_function(function_type)
    }
}
