use std::sync::RwLock;

use zwave_core::prelude::*;

#[derive(Default)]
/// Internal storage for the driver instance. Since the driver is meant be used from external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct DriverStorage {
    node_id_type: RwLock<NodeIdType>,
}

impl DriverStorage {
    pub fn node_id_type(&self) -> NodeIdType {
        *self.node_id_type.read().unwrap()
    }

    pub fn set_node_id_type(&self, node_id_type: NodeIdType) {
        *self.node_id_type.write().unwrap() = node_id_type;
    }
}
