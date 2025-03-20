use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use zwave_core::{cache::CacheValue, prelude::*, value_id::EndpointValueId};

/// Storage shared between the Serial API and driver actors, containing information
/// that is needed to correctly parse and serialize commands.
pub(crate) struct SerialApiStorage {
    own_node_id: RwLock<NodeId>,
    node_id_type: RwLock<NodeIdType>,
    sdk_version: RwLock<Option<Version>>,
}

impl SerialApiStorage {
    pub fn new(node_id_type: NodeIdType) -> Self {
        Self {
            own_node_id: RwLock::new(NodeId::unspecified()),
            node_id_type: RwLock::new(node_id_type),
            sdk_version: RwLock::new(None),
        }
    }

    pub fn own_node_id(&self) -> NodeId {
        *self.own_node_id.read().unwrap()
    }

    pub fn set_own_node_id(&self, own_node_id: NodeId) {
        *self.own_node_id.write().unwrap() = own_node_id;
    }

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
}
