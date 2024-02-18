use bytes::BytesMut;
use crate::serialize::{self, Serializable};
use crate::parse::{bytes::be_u8, combinators::map_res};
use crate::prelude::*;
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum NodeIdType {
    #[default]
    NodeId8Bit = 0x01,
    NodeId16Bit = 0x02,
}

impl Display for NodeIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeIdType::NodeId8Bit => write!(f, "8 bit"),
            NodeIdType::NodeId16Bit => write!(f, "16 bit"),
        }
    }
}

impl Parsable for NodeIdType {
    fn parse(i: &mut bytes::Bytes) -> crate::parse::ParseResult<Self> {
        map_res(be_u8, NodeIdType::try_from).parse(i)
    }
}

impl Serializable for NodeIdType {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output)
    }
}
