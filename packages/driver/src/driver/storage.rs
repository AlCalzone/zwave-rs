use std::collections::HashMap;
use zwave_core::{
    cache::CacheValue,
    security::{SecurityManager, SecurityManager2},
    util::Locked,
    value_id::EndpointValueId,
};

/// Internal storage for the driver instance and shared API instances.
/// Since the driver is meant be used from external (application) code,
/// in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring
/// a mutable reference.
pub(crate) struct DriverStorage {
    value_cache: Locked<HashMap<EndpointValueId, CacheValue>>,
    security_manager: Locked<Option<SecurityManager>>,
    security_manager2: Locked<Option<SecurityManager2>>,
}

impl DriverStorage {
    pub fn new() -> Self {
        Self {
            value_cache: Locked::new(HashMap::new()),
            security_manager: Locked::new(None),
            security_manager2: Locked::new(None),
        }
    }

    pub(crate) fn value_cache(&self) -> &Locked<HashMap<EndpointValueId, CacheValue>> {
        &self.value_cache
    }

    pub(crate) fn security_manager(&self) -> &Locked<Option<SecurityManager>> {
        &self.security_manager
    }

    pub(crate) fn security_manager2(&self) -> &Locked<Option<SecurityManager2>> {
        &self.security_manager2
    }
}
