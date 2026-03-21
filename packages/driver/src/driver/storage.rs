use alloc::collections::BTreeMap;
use zwave_core::{
    cache::CacheValue,
    security::{SecurityManager, SecurityManager2},
    util::Locked,
    value_id::EndpointValueId,
};

/// Internal storage for the driver instance and shared API instances.
pub(crate) struct DriverStorage {
    value_cache: Locked<BTreeMap<EndpointValueId, CacheValue>>,
    security_manager: Locked<Option<SecurityManager>>,
    security_manager2: Locked<Option<SecurityManager2>>,
}

impl DriverStorage {
    pub fn new() -> Self {
        Self {
            value_cache: Locked::new(BTreeMap::new()),
            security_manager: Locked::new(None),
            security_manager2: Locked::new(None),
        }
    }

    pub(crate) fn value_cache(&self) -> &Locked<BTreeMap<EndpointValueId, CacheValue>> {
        &self.value_cache
    }

    pub(crate) fn security_manager(&self) -> &Locked<Option<SecurityManager>> {
        &self.security_manager
    }

    pub(crate) fn security_manager2(&self) -> &Locked<Option<SecurityManager2>> {
        &self.security_manager2
    }
}
