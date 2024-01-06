use crate::prelude::*;
use crate::encoding;
use proc_macros::TryFromRepr;

use cookie_factory as cf;
use nom::{
    bits::complete::take as take_bits, combinator::map_res, error::context,
    number::complete::be_u8,
};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum ProtocolVersion {
    V2 = 1,
    V5 = 2,
    V6 = 3,
}

impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V2 => write!(f, "Z-Wave v2.0"),
            Self::V5 => write!(f, "ZDK 4.2x, ZDK 5.0x"),
            Self::V6 => write!(f, "ZDK 4.5x, ZDK 6.0x"),
        }
    }
}

impl NomTryFromPrimitive for ProtocolVersion {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown protocol version: {:#04x}", repr)
    }
}

impl Parsable for ProtocolVersion {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ProtocolVersion",
            map_res(be_u8, ProtocolVersion::try_from_primitive),
        )(i)
    }
}

impl BitParsable for ProtocolVersion {
    fn parse(i: encoding::BitInput) -> encoding::BitParseResult<Self> {
        context(
            "ProtocolVersion",
            map_res(take_bits(3usize), |x: u8| {
                ProtocolVersion::try_from_primitive(x)
            }),
        )(i)
    }
}

impl Serializable for ProtocolVersion {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
