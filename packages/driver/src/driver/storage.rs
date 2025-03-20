use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use zwave_core::{cache::CacheValue, prelude::*, security::{SecurityManager, SecurityManagerStorage}, value_id::EndpointValueId};

/// Internal storage for the driver instance and shared API instances.
/// Since the driver is meant be used from external (application) code,
/// in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring
/// a mutable reference.
pub(crate) struct DriverStorage {
    value_cache: RwLock<HashMap<EndpointValueId, CacheValue>>,
    security_manager: RwLock<Option<SecurityManager>>,
}

impl DriverStorage {
    pub fn new() -> Self {
        Self {
            value_cache: RwLock::new(HashMap::new()),
            security_manager: RwLock::new(None),
        }
    }

    pub fn value_cache(&self) -> RwLockReadGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.read().unwrap()
    }

    pub fn value_cache_mut(&self) -> RwLockWriteGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.write().unwrap()
    }

    pub fn security_manager(&self) -> RwLockReadGuard<Option<SecurityManager>> {
        self.security_manager.read().unwrap()
    }

    pub fn security_manager_mut(&self) -> RwLockWriteGuard<Option<SecurityManager>> {
        self.security_manager.write().unwrap()
    }
}
