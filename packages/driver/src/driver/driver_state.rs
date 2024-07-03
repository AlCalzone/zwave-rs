use crate::{ControllerStorage, NodeStorage};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use zwave_core::{
    definitions::{FunctionType, NodeId},
    security::SecurityManager,
};

/// The driver can be in one of multiple states, each of which has a different set of capabilities.
pub trait DriverState: Clone + Sync + Send {
    /// Whether the driver supports executing the given function type in this phase
    #[allow(unused_variables)]
    fn supports_function(&self, function_type: FunctionType) -> bool {
        // By default: Don't know, don't care
        false
    }

    /// Access the security manager instance if it is already set up
    fn security_manager(&self) -> Option<Arc<RwLock<SecurityManager>>> {
        None
    }
}

/// The driver isn't fully initialized yet
#[derive(Clone)]
pub struct Init;
impl DriverState for Init {}

/// The driver is ready to use normally
#[derive(Clone)]
pub struct Ready {
    pub(crate) controller: Arc<ControllerStorage>,
    pub(crate) nodes: Arc<BTreeMap<NodeId, NodeStorage>>,
    pub(crate) security_manager: Option<Arc<RwLock<SecurityManager>>>,
}

impl DriverState for Ready {
    fn supports_function(&self, function_type: FunctionType) -> bool {
        self.controller
            .supported_function_types
            .contains(&function_type)
    }

    fn security_manager(&self) -> Option<Arc<RwLock<SecurityManager>>> {
        self.security_manager.clone()
    }
}
