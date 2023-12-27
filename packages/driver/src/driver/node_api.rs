use super::Ready;
use crate::NodeStorage;
use crate::{Driver, Node};
use zwave_core::definitions::*;

// API for node instances
impl Driver<Ready> {
    pub fn get_node(&self, node_id: &NodeId) -> Option<Node> {
        // Do not return a node API for the Serial API controller
        if node_id == &self.controller().own_node_id() {
            return None;
        }

        self.state.nodes.get(node_id).map(|storage| {
            Node::new(
                *node_id,
                // We clone the protocol data from storage to avoid lots of node methods
                // needing an Option as the return type in case the node was removed after
                // the call to get_node
                storage.protocol_data.clone(),
                self,
            )
        })
    }

    pub fn nodes(&self) -> impl Iterator<Item = Node> {
        self.state
            .nodes
            .keys()
            .filter_map(|node_id| self.get_node(node_id))
    }

    pub(crate) fn get_node_storage(&self, node_id: &NodeId) -> Option<&NodeStorage> {
        self.state.nodes.get(node_id)
    }
}
