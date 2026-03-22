use zwave_pal::prelude::*;
use super::{Controller, Ready};
use crate::Node;
use zwave_core::prelude::*;

// FIXME: We should have a wrapper to expose only supported commands to lib users

// API for node instances
impl Controller<'_, Ready> {
    pub fn node(&self, node_id: NodeId) -> Option<Node<'_>> {
        // Do not return a node API for the Serial API controller
        if node_id == self.own_node_id() {
            return None;
        }

        self.node_state(node_id).protocol_data().map(|protocol_data| {
            Node::new(
                node_id,
                protocol_data,
                self,
            )
        })
    }

    pub fn nodes(&self) -> Vec<Node<'_>> {
        self.state.nodes.inspect(|nodes| {
            nodes.keys()
                .filter_map(move |node_id| self.node(*node_id))
                .collect()
        })
    }
}
