use crate::{driver::cache::ValueCache, Node};
use zwave_core::{
    cache::{Cache, CacheValue},
    definitions::NodeId,
    value_id::ValueId,
};

pub struct NodeValueCache<'a> {
    node_id: NodeId,
    driver_value_cache: ValueCache<'a>,
}

impl<'a> NodeValueCache<'a> {}

impl Cache<ValueId> for NodeValueCache<'_> {
    fn read(&self, key: &ValueId) -> Option<CacheValue> {
        self.driver_value_cache
            .read(&key.with_node_id(&self.node_id))
    }

    fn write(&mut self, key: &ValueId, value: CacheValue) {
        self.driver_value_cache
            .write(&key.with_node_id(&self.node_id), value);
    }

    fn delete(&mut self, key: &ValueId) {
        self.driver_value_cache
            .delete(&key.with_node_id(&self.node_id));
    }
}

impl<'a> Node<'a> {
    pub fn value_cache(&self) -> NodeValueCache<'a> {
        NodeValueCache {
            node_id: self.id,
            driver_value_cache: self.driver.value_cache(),
        }
    }
}
