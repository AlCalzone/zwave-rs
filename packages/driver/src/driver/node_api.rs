use super::Ready;
use crate::Driver;
use crate::{InterviewStage, NodeStorage};
use zwave_core::definitions::*;

// Defines methods used by Node instances
impl Driver<Ready> {
    fn get_node_storage(&self, node_id: &NodeId) -> Option<&NodeStorage> {
        self.state.nodes.get(node_id)
    }

    pub(crate) fn get_node_interview_stage(&self, node_id: &NodeId) -> Option<InterviewStage> {
        self.get_node_storage(node_id)
            .map(|storage| *storage.interview_stage.read().unwrap())
    }

    pub(crate) fn set_node_interview_stage(
        &self,
        node_id: &NodeId,
        interview_stage: InterviewStage,
    ) {
        if let Some(storage) = self.get_node_storage(node_id) {
            *storage.interview_stage.write().unwrap() = interview_stage;
        }
    }
}
