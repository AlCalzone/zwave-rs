use crate::encoding;
use crate::encoding::WriteLastNBits;
use crate::munch::{bits, combinators::map_res};
use crate::prelude::*;
use bytes::Bytes;
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum NodeType {
    Controller = 0,
    EndNode = 1,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Controller => write!(f, "Controller"),
            NodeType::EndNode => write!(f, "End Node"),
        }
    }
}

impl BitParsable for NodeType {
    fn parse(i: &mut (Bytes, usize)) -> crate::munch::ParseResult<Self> {
        map_res(bits::take(1usize), |x: u8| NodeType::try_from(x)).parse(i)
    }
}

impl BitSerializable for NodeType {
    fn write(&self, b: &mut encoding::BitOutput) {
        b.write_last_n_bits((*self) as u8, 1);
    }
}
