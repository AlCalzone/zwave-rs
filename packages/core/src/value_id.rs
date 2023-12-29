use typed_builder::TypedBuilder;

use crate::prelude::*;

/// Uniquely identifies which CC and property a value belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypedBuilder)]
pub struct ValueId {
    command_class: CommandClasses,
    #[builder(setter(into))]
    property: u32,
    #[builder(default, setter(strip_option))]
    property_key: Option<u32>,
}

impl ValueId {
    pub fn new(
        command_class: CommandClasses,
        property: impl Into<u32>,
        property_key: Option<u32>,
    ) -> Self {
        Self {
            command_class,
            property: property.into(),
            property_key,
        }
    }

    pub fn command_class(&self) -> CommandClasses {
        self.command_class
    }

    pub fn property(&self) -> u32 {
        self.property
    }

    pub fn property_key(&self) -> Option<u32> {
        self.property_key
    }

    pub fn with_node_id(&self, node_id: &NodeId) -> EndpointValueId {
        EndpointValueId {
            node_id: *node_id,
            endpoint: EndpointIndex::Root,
            value_id: *self,
        }
    }
}

/// Uniquely identifies which Node, endpoint, CC and property a value belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndpointValueId {
    node_id: NodeId,
    endpoint: EndpointIndex,
    value_id: ValueId,
}

impl EndpointValueId {
    pub fn new(node_id: NodeId, endpoint: EndpointIndex, value_id: ValueId) -> Self {
        Self {
            node_id,
            endpoint,
            value_id,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn with_endpoint(&self, endpoint: EndpointIndex) -> Self {
        Self { endpoint, ..*self }
    }

    pub fn endpoint(&self) -> EndpointIndex {
        self.endpoint
    }

    pub fn command_class(&self) -> CommandClasses {
        self.value_id.command_class
    }

    pub fn property(&self) -> u32 {
        self.value_id.property
    }

    pub fn property_key(&self) -> Option<u32> {
        self.value_id.property_key
    }
}
