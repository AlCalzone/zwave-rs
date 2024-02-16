use crate::{
    munch::bytes::{be_u16, be_u8},
    prelude::*,
};
use cookie_factory as cf;
use std::fmt::{Debug, Display};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u16);

pub const NODE_ID_UNSPECIFIED: NodeId = NodeId(0);
pub const NODE_ID_BROADCAST: NodeId = NodeId(0xff);

impl NodeId {
    pub fn new<T>(id: T) -> Self
    where
        T: Into<u16>,
    {
        Self(id.into())
    }

    pub fn broadcast() -> Self {
        NODE_ID_BROADCAST
    }

    pub fn unspecified() -> Self {
        NODE_ID_UNSPECIFIED
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>3}", self.0)
    }
}

macro_rules! impl_conversions_for {
    ($t:ty) => {
        impl From<$t> for NodeId {
            fn from(val: $t) -> Self {
                Self(val as u16)
            }
        }

        impl From<NodeId> for $t {
            fn from(val: NodeId) -> Self {
                val.0 as $t
            }
        }

        impl PartialEq<$t> for NodeId {
            fn eq(&self, other: &$t) -> bool {
                self == &NodeId::from(*other)
            }
        }

        impl PartialOrd<$t> for NodeId {
            fn partial_cmp(&self, other: &$t) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&NodeId::from(*other))
            }
        }
    };
}

impl_conversions_for!(u8);
impl_conversions_for!(u16);
impl_conversions_for!(i32);

impl NodeId {
    pub fn parse(
        i: &mut bytes::Bytes,
        node_id_type: NodeIdType,
    ) -> crate::munch::ParseResult<Self> {
        match node_id_type {
            NodeIdType::NodeId8Bit => {
                let node_id = be_u8().parse(i)?;
                Ok(Self(node_id as u16))
            }
            NodeIdType::NodeId16Bit => {
                let node_id = be_u16().parse(i)?;
                Ok(Self(node_id))
            }
        }
    }

    pub fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        node_id_type: NodeIdType,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::{be_u16, be_u8};
        move |out| match node_id_type {
            NodeIdType::NodeId8Bit => be_u8(self.0 as u8)(out),
            NodeIdType::NodeId16Bit => be_u16(self.0)(out),
        }
    }
}
