use crate::prelude::*;
use typed_builder::TypedBuilder;

/// Uniquely identifies which CC and property a value belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypedBuilder)]
pub struct ValueId {
    command_class: CommandClasses,
    #[builder(setter(into))]
    property: u32,
    #[builder(default, setter(into))]
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

/// A subset of [ValueId] used for matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueIdProperties {
    property: u32,
    property_key: Option<u32>,
}

impl ValueIdProperties {
    pub fn new(property: impl Into<u32>, property_key: Option<u32>) -> Self {
        Self {
            property: property.into(),
            property_key,
        }
    }

    pub fn property(&self) -> u32 {
        self.property
    }

    pub fn property_key(&self) -> Option<u32> {
        self.property_key
    }

    pub fn with_cc(&self, cc: CommandClasses) -> ValueId {
        ValueId::new(cc, self.property, self.property_key)
    }
}

impl From<ValueId> for ValueIdProperties {
    fn from(value: ValueId) -> Self {
        Self {
            property: value.property,
            property_key: value.property_key,
        }
    }
}

impl From<(u32, Option<u32>)> for ValueIdProperties {
    fn from(value: (u32, Option<u32>)) -> Self {
        Self {
            property: value.0,
            property_key: value.1,
        }
    }
}
