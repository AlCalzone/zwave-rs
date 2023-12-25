use std::collections::BTreeMap;

use zwave_core::definitions::{FunctionType, NodeId};

use crate::{Controller, Node};

/// The driver can be in one of multiple states, each of which has a different set of capabilities.
pub trait DriverState {
    /// An immutable reference to the controller, if available
    fn controller(&self) -> Option<&Controller>;

    /// A mutable reference to the controller, if available
    fn controller_mut(&mut self) -> Option<&mut Controller>;

    /// Whether the driver supports executing the given function type in this phase
    #[allow(unused_variables)]
    fn supports_function(&self, function_type: FunctionType) -> bool {
        // By default: Don't know, don't care
        false
    }
}

/// The driver isn't fully initialized yet
pub struct Init;

impl DriverState for Init {
    fn controller(&self) -> Option<&Controller> {
        None
    }

    fn controller_mut(&mut self) -> Option<&mut Controller> {
        None
    }
}

/// The driver is ready to use normally
#[derive(Debug)]
pub struct Ready {
    pub(crate) controller: Controller,
    pub(crate) nodes: BTreeMap<NodeId, Node>,
}

impl DriverState for Ready {
    fn controller(&self) -> Option<&Controller> {
        Some(&self.controller)
    }

    fn controller_mut(&mut self) -> Option<&mut Controller> {
        Some(&mut self.controller)
    }

    fn supports_function(&self, function_type: FunctionType) -> bool {
        self.controller.supports_function(function_type)
    }
}
