use zwave_core::definitions::NodeInformationProtocolData;

use crate::InterviewStage;
use std::sync::RwLock;


#[derive(Debug)]
/// Internal storage for a node instance. Since this is meant be used from both library and external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct NodeStorage {
    pub(crate) interview_stage: RwLock<InterviewStage>,
    pub(crate) protocol_data: NodeInformationProtocolData,
}

impl NodeStorage {
    pub fn new(protocol_data: NodeInformationProtocolData) -> Self {
        Self {
            interview_stage: RwLock::new(InterviewStage::None),
            protocol_data
        }
    }
}
