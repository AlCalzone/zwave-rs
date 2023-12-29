use zwave_core::{
    cache::{Cache, CacheValue},
    value_id::NodeValueId,
};

use crate::{Driver, Ready};

pub struct ValueCache<'a> {
    driver: &'a Driver<Ready>,
}

impl<'a> ValueCache<'a> {}

impl Cache<NodeValueId> for ValueCache<'_> {
    fn read(&self, key: &NodeValueId) -> Option<CacheValue> {
        self.driver.storage.value_cache().get(key).cloned()
    }

    fn write(&mut self, key: &NodeValueId, value: CacheValue) {
        self.driver.storage.value_cache_mut().insert(*key, value);
    }

    fn delete(&mut self, key: &NodeValueId) {
        self.driver.storage.value_cache_mut().remove(key);
    }
}

impl Driver<Ready> {
    pub fn value_cache(&self) -> ValueCache {
        ValueCache { driver: self }
    }
}
