use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use zwave_core::{cache::CacheValue, prelude::*, value_id::EndpointValueId};

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

#[derive(Default)]
/// Internal storage for the driver instance which is shared between tasks.
/// This is meant to be used both from the tasks and the public API.
pub(crate) struct DriverStorageShared {
    value_cache: RwLock<HashMap<EndpointValueId, CacheValue>>,
}

impl DriverStorageShared {
    pub fn value_cache(&self) -> RwLockReadGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.read().unwrap()
    }

    pub fn value_cache_mut(&self) -> RwLockWriteGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.write().unwrap()
    }
}
