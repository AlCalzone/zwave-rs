use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use zwave_core::{
    cache::CacheValue, prelude::*, security::SecurityManager, value_id::EndpointValueId,
};
use zwave_logging::loggers::{controller::ControllerLogger, driver::DriverLogger};

use crate::{BackgroundLogger, DriverOptions};

use super::DriverOptionsStatic;

/// Internal storage for the driver instance. Since the driver is meant be used from external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct DriverStorage {
    options: DriverOptionsStatic,
    driver_logger: DriverLogger,
    controller_logger: ControllerLogger,
}

impl DriverStorage {
    pub fn new(
        options: DriverOptionsStatic,
        driver_logger: DriverLogger,
        controller_logger: ControllerLogger,
    ) -> Self {
        Self {
            options,
            driver_logger,
            controller_logger,
        }
    }

    pub fn options(&self) -> &DriverOptionsStatic {
        &self.options
    }

    pub fn driver_logger(&self) -> &DriverLogger {
        &self.driver_logger
    }

    pub fn controller_logger(&self) -> &ControllerLogger {
        &self.controller_logger
    }
}

/// Internal storage for the driver instance which is shared between tasks.
/// This is meant to be used both from the tasks and the public API.
pub(crate) struct DriverStorageShared {
    // The shared logger used by all specific logger instances
    logger: Arc<BackgroundLogger>,
    value_cache: RwLock<HashMap<EndpointValueId, CacheValue>>,
    own_node_id: RwLock<NodeId>,
    node_id_type: RwLock<NodeIdType>,
    sdk_version: RwLock<Option<Version>>,
}

impl DriverStorageShared {
    pub fn new(logger: Arc<BackgroundLogger>, node_id_type: NodeIdType) -> Self {
        Self {
            logger,
            value_cache: RwLock::new(HashMap::new()),
            own_node_id: RwLock::new(NodeId::unspecified()),
            node_id_type: RwLock::new(node_id_type),
            sdk_version: RwLock::new(None),
        }
    }

    pub fn value_cache(&self) -> RwLockReadGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.read().unwrap()
    }

    pub fn value_cache_mut(&self) -> RwLockWriteGuard<HashMap<EndpointValueId, CacheValue>> {
        self.value_cache.write().unwrap()
    }

    pub fn logger(&self) -> &Arc<BackgroundLogger> {
        &self.logger
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
