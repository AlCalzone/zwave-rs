use crate::{driver::cache::ValueCache, EndpointLike};
use zwave_core::{
    cache::{Cache, CacheValue},
    value_id::{EndpointValueId, ValueId},
};

pub struct EndpointValueCache<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
    driver_value_cache: ValueCache<'a>,
}

impl<'a> EndpointValueCache<'a> {
    pub fn new(endpoint: &'a dyn EndpointLike<'a>, driver_value_cache: ValueCache<'a>) -> Self {
        Self {
            endpoint,
            driver_value_cache,
        }
    }

    fn get_value_id(&self, value_id: &ValueId) -> EndpointValueId {
        EndpointValueId::new(self.endpoint.node_id(), self.endpoint.index(), *value_id)
    }
}

impl Cache<ValueId> for EndpointValueCache<'_> {
    fn read(&self, key: &ValueId) -> Option<CacheValue> {
        self.driver_value_cache.read(&self.get_value_id(key))
    }

    fn write(&mut self, key: &ValueId, value: CacheValue) {
        self.driver_value_cache
            .write(&self.get_value_id(key), value);
    }

    fn write_many(&mut self, values: impl Iterator<Item = (ValueId, CacheValue)>) {
        let key_value_pairs: Vec<_> = values
            .map(|(key, value)| (self.get_value_id(&key), value))
            .collect();
        self.driver_value_cache
            .write_many(key_value_pairs.into_iter());
    }

    fn delete(&mut self, key: &ValueId) {
        self.driver_value_cache.delete(&self.get_value_id(key));
    }
}
