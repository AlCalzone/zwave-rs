use crate::encoding::WriteLastNBits;
use crate::prelude::*;
use crate::encoding;
use proc_macros::TryFromRepr;

use nom::{bits::complete::take as take_bits, combinator::map_res};
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

impl NomTryFromPrimitive for NodeType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown node type: {:#04x}", repr)
    }
}

impl BitParsable for NodeType {
    fn parse(i: encoding::BitInput) -> encoding::BitParseResult<Self> {
        map_res(take_bits(1usize), |x: u8| NodeType::try_from_primitive(x))(i)
    }
}

impl BitSerializable for NodeType {
    fn write(&self, b: &mut encoding::BitOutput) {
        b.write_last_n_bits((*self) as u8, 1);
    }
}
