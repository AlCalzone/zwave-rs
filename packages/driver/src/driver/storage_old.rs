use crate::{ControllerStorage, EndpointStorage, NodeStorage};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use zwave_core::{
    cache::CacheValue, prelude::*, security::SecurityManagerStorage, value_id::EndpointValueId,
};

/// Internal storage for the driver instance and shared API instances.
/// Since the driver is meant be used from external (application) code,
/// in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring
/// a mutable reference.
pub(crate) struct DriverStorage {
    value_cache: RwLock<HashMap<EndpointValueId, CacheValue>>,

    // own_node_id: RwLock<NodeId>,
    node_id_type: RwLock<NodeIdType>,
    sdk_version: RwLock<Option<Version>>,

    controller: RwLock<Option<ControllerStorage>>,
    nodes: RwLock<BTreeMap<NodeId, NodeStorage>>,
    endpoints: RwLock<BTreeMap<(NodeId, EndpointIndex), EndpointStorage>>,

    security_manager: RwLock<Option<Arc<SecurityManagerStorage>>>,
}

impl DriverStorage {
    pub fn new(node_id_type: NodeIdType) -> Self {
        Self {
            value_cache: RwLock::new(HashMap::new()),
            // own_node_id: RwLock::new(NodeId::unspecified()),
            node_id_type: RwLock::new(node_id_type),
            sdk_version: RwLock::new(None),

            controller: RwLock::new(None),
            nodes: RwLock::new(BTreeMap::new()),
            endpoints: RwLock::new(BTreeMap::new()),

            security_manager: RwLock::new(None),
        }
    }

    pub fn value_cache(&self) -> RwLockReadGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.read().unwrap()
    }

    pub fn value_cache_mut(&self) -> RwLockWriteGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.write().unwrap()
    }

    // pub fn own_node_id(&self) -> NodeId {
    //     *self.own_node_id.read().unwrap()
    // }

    // pub fn set_own_node_id(&self, own_node_id: NodeId) {
    //     *self.own_node_id.write().unwrap() = own_node_id;
    // }

    pub fn node_id_type(&self) -> NodeIdType {
        *self.node_id_type.read().unwrap()
    }

    pub fn set_node_id_type(&self, node_id_type: NodeIdType) {
        *self.node_id_type.write().unwrap() = node_id_type;
    }

    pub fn sdk_version(&self) -> Option<Version> {
        *self.sdk_version.read().unwrap()
    }

    pub fn set_sdk_version(&self, version: Version) {
        *self.sdk_version.write().unwrap() = Some(version);
    }

    pub fn nodes(&self) -> RwLockReadGuard<BTreeMap<NodeId, NodeStorage>> {
        self.nodes.read().unwrap()
    }

    pub fn nodes_mut(&self) -> RwLockWriteGuard<BTreeMap<NodeId, NodeStorage>> {
        self.nodes.write().unwrap()
    }

    pub fn endpoints(&self) -> RwLockReadGuard<BTreeMap<(NodeId, EndpointIndex), EndpointStorage>> {
        self.endpoints.read().unwrap()
    }

    pub fn endpoints_mut(
        &self,
    ) -> RwLockWriteGuard<BTreeMap<(NodeId, EndpointIndex), EndpointStorage>> {
        self.endpoints.write().unwrap()
    }

    pub fn controller(&self) -> RwLockReadGuard<Option<ControllerStorage>> {
        self.controller.read().unwrap()
    }

    pub fn controller_mut(&self) -> RwLockWriteGuard<Option<ControllerStorage>> {
        self.controller.write().unwrap()
    }

    pub fn security_manager(&self) -> RwLockReadGuard<Option<Arc<SecurityManagerStorage>>> {
        self.security_manager.read().unwrap()
    }

    pub fn security_manager_mut(&self) -> RwLockWriteGuard<Option<Arc<SecurityManagerStorage>>> {
        self.security_manager.write().unwrap()
    }
}
