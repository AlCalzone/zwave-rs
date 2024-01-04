use zwave_core::definitions::*;

use crate::InterviewStage;
use std::{collections::BTreeMap, sync::RwLock};

#[derive(Debug)]
/// Internal storage for a node instance. Since this is meant be used from both library and external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct NodeStorage {
    pub(crate) interview_stage: RwLock<InterviewStage>,
    pub(crate) protocol_data: NodeInformationProtocolData,
    pub(crate) endpoints: BTreeMap<EndpointIndex, EndpointStorage>,
}

impl NodeStorage {
    pub fn new(protocol_data: NodeInformationProtocolData) -> Self {
        let mut endpoints = BTreeMap::new();
        // Always add the root endpoint
        endpoints.insert(EndpointIndex::Root, EndpointStorage::new());

        Self {
            interview_stage: RwLock::new(InterviewStage::None),
            protocol_data,
            endpoints,
        }
    }
}

#[derive(Debug)]
/// Internal storage for an endpoint instance. Since this is meant be used from both library and external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct EndpointStorage {
    pub(crate) cc_info: RwLock<BTreeMap<CommandClasses, CommandClassInfo>>,
}

impl EndpointStorage {
    pub fn new() -> Self {
        Self {
            cc_info: RwLock::new(BTreeMap::new()),
        }
    }
}
