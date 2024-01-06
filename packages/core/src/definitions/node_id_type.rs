use crate::prelude::*;
use crate::encoding;
use proc_macros::TryFromRepr;

use cookie_factory as cf;
use nom::{combinator::map_res, number::complete::be_u8};
use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum NodeIdType {
    #[default]
    NodeId8Bit = 0x01,
    NodeId16Bit = 0x02,
}

impl NomTryFromPrimitive for NodeIdType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown node ID type: {:#04x}", repr)
    }
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
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        map_res(be_u8, NodeIdType::try_from_primitive)(i)
    }
}

impl Serializable for NodeIdType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8((*self) as u8)
    }
}
