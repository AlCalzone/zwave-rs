use super::Ready;
use crate::NodeStorage;
use crate::{Driver, Node};
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

    pub(crate) fn get_node_storage(&self, node_id: &NodeId) -> Option<&NodeStorage> {
        self.state.nodes.get(node_id)
    }
}
