use crate::parse::{
    bits::take as take_bits,
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::{BytesMut, Bytes};
use crate::serialize::{self, Serializable};
use proc_macros::TryFromRepr;
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

impl Parsable for ProtocolVersion {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        context("ProtocolVersion", map_res(be_u8, ProtocolVersion::try_from)).parse(i)
    }
}

impl BitParsable for ProtocolVersion {
    fn parse(i: &mut (Bytes, usize)) -> crate::parse::ParseResult<Self> {
        context(
            "ProtocolVersion",
            map_res(take_bits(3usize), |x: u8| ProtocolVersion::try_from(x)),
        )
        .parse(i)
    }
}

impl Serializable for ProtocolVersion {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output)
    }
}
