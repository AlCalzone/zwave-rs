use super::Ready;
use crate::{Driver, Node};
use crate::{InterviewStage, NodeStorage};
use zwave_core::definitions::*;

// API for node instances
impl Driver<Ready> {
    pub fn get_node(&self, node_id: &NodeId) -> Option<Node> {
        if self.state.nodes.contains_key(node_id) {
            Some(Node::new(*node_id, self))
        } else {
            None
        }
    }

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
