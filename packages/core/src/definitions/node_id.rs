use super::NodeIdType;
use crate::encoding;
use cookie_factory as cf;
use nom::number::complete::{be_u16, be_u8};
use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(u16);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>3}", self.0)
    }
}

impl From<u8> for NodeId {
    fn from(val: u8) -> Self {
        Self(val as u16)
    }
}

impl From<u16> for NodeId {
    fn from(val: u16) -> Self {
        Self(val)
    }
}

impl NodeId {
    pub fn parse(i: encoding::Input, node_id_type: NodeIdType) -> encoding::ParseResult<Self> {
        match node_id_type {
            NodeIdType::NodeId8Bit => {
                let (i, node_id) = be_u8(i)?;
                Ok((i, Self(node_id as u16)))
            }
            NodeIdType::NodeId16Bit => {
                let (i, node_id) = be_u16(i)?;
                Ok((i, Self(node_id)))
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
