use typed_builder::TypedBuilder;

use crate::prelude::*;

/// Uniquely identifies which CC, endpoint and property a value belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypedBuilder)]
pub struct ValueId {
    command_class: CommandClasses,
    #[builder(default)]
    endpoint: EndpointIndex,
    property: u32,
    #[builder(default, setter(strip_option))]
    property_key: Option<u32>,
}

impl ValueId {
    pub fn command_class(&self) -> CommandClasses {
        self.command_class
    }

    pub fn endpoint(&self) -> EndpointIndex {
        self.endpoint
    }

    pub fn property(&self) -> u32 {
        self.property
    }

    pub fn property_key(&self) -> Option<u32> {
        self.property_key
    }

    pub fn with_node_id(&self, node_id: &NodeId) -> NodeValueId {
        NodeValueId {
            node_id: *node_id,
            value_id: *self,
        }
    }
}

/// Uniquely identifies which Node, CC, endpoint and property a value belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeValueId {
    node_id: NodeId,
    value_id: ValueId,
}

impl NodeValueId {
    pub fn new(node_id: NodeId, value_id: ValueId) -> Self {
        Self {
            node_id,
            value_id,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn command_class(&self) -> CommandClasses {
        self.value_id.command_class
    }

    pub fn endpoint(&self) -> EndpointIndex {
        self.value_id.endpoint
    }

    pub fn property(&self) -> u32 {
        self.value_id.property
    }

    pub fn property_key(&self) -> Option<u32> {
        self.value_id.property_key
    }
}

