use super::storage::DriverStorage;
use crate::{driver_api::DriverApi, Ready};
use std::sync::Arc;
use zwave_core::{
    cache::{Cache, CacheValue},
    value_id::EndpointValueId,
};

pub struct ValueCache<'a> {
    storage: &'a Arc<DriverStorage>,
}

impl<'a> ValueCache<'a> {
    pub(crate) fn new(storage: &'a Arc<DriverStorage>) -> Self {
        Self { storage }
    }
}

impl Cache<EndpointValueId> for ValueCache<'_> {
    fn read(&self, key: &EndpointValueId) -> Option<CacheValue> {
        self.storage.value_cache().get(key).cloned()
    }

    fn write(&mut self, key: &EndpointValueId, value: CacheValue) {
        self.storage.value_cache_mut().insert(*key, value);
    }

    fn write_many(&mut self, values: impl Iterator<Item = (EndpointValueId, CacheValue)>) {
        self.storage.value_cache_mut().extend(values);
    }

    fn delete(&mut self, key: &EndpointValueId) {
        self.storage.value_cache_mut().remove(key);
    }
}

impl DriverApi<Ready> {
    pub fn value_cache(&self) -> ValueCache<'_> {
        ValueCache::new(&self.storage)
    }
}
