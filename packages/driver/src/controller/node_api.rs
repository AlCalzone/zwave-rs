use super::{Controller, Ready};
use crate::Node;
use zwave_core::prelude::*;

// FIXME: We should have a wrapper to expose only supported commands to lib users

// API for node instances
impl Controller<'_, Ready> {
    pub fn get_node(&self, node_id: &NodeId) -> Option<Node> {
        // Do not return a node API for the Serial API controller
        if node_id == &self.own_node_id() {
            return None;
        }

        self.state
            .nodes
            .read()
            .expect("failed to lock node storage for reading")
            .get(node_id)
            .map(|storage| {
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

    pub fn nodes(&self) -> Vec<Node> {
        self.node_storage()
            .keys()
            .filter_map(move |node_id| self.get_node(node_id))
            .collect()
    }
}
