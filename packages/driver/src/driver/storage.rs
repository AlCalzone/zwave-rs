use std::{sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}, collections::HashMap};

use zwave_core::{prelude::*, value_id::NodeValueId, cache::CacheValue};

#[derive(Default)]
/// Internal storage for the driver instance. Since the driver is meant be used from external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct DriverStorage {
    node_id_type: RwLock<NodeIdType>,
    value_cache: RwLock<HashMap<NodeValueId, CacheValue>>,
}

impl DriverStorage {
    pub fn node_id_type(&self) -> NodeIdType {
        *self.node_id_type.read().unwrap()
    }

    pub fn set_node_id_type(&self, node_id_type: NodeIdType) {
        *self.node_id_type.write().unwrap() = node_id_type;
    }

    pub fn value_cache(&self) -> RwLockReadGuard<HashMap<NodeValueId, CacheValue>> {
        self.value_cache.read().unwrap()
    }

    pub fn value_cache_mut(&self) -> RwLockWriteGuard<HashMap<NodeValueId, CacheValue>> {
        self.value_cache.write().unwrap()
    }
}
