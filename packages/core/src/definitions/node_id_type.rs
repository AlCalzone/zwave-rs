use crate::encoding::{self, BitParsable, BitSerializable, WriteLastNBits};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use encoding::{Parsable, Serializable};
use nom::{combinator::map, number::complete::be_u8};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum NodeIdType {
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
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        map(be_u8, |x: u8| NodeIdType::try_from(x).unwrap())(i)
    }
}

impl Serializable for NodeIdType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8((*self) as u8)
    }
}
