use zwave_core::definitions::FunctionType;

use crate::Controller;

/// The driver can be in one of multiple phases. Each phase has a different set of capabilities.
pub trait DriverPhase {
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

pub struct Init;

impl DriverPhase for Init {
    fn controller(&self) -> Option<&Controller> {
        None
    }

    fn controller_mut(&mut self) -> Option<&mut Controller> {
        None
    }
}

pub struct Ready {
    pub(crate) controller: Controller,
}

impl DriverPhase for Ready {
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

pub struct Destroyed;

impl DriverPhase for Destroyed {
    fn controller(&self) -> Option<&Controller> {
        None
    }

    fn controller_mut(&mut self) -> Option<&mut Controller> {
        None
    }
}
