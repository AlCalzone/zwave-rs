use crate::encoding::{self, NomTryFromPrimitive, Parsable, Serializable};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use nom::{combinator::map_res, error::context, number::complete::be_u8};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ProtocolType {
    ZWave,
    ZWaveAV,
    ZWaveIP,
}

impl Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::ZWave => write!(f, "Z-Wave"),
            ProtocolType::ZWaveAV => write!(f, "Z-Wave AV"),
            ProtocolType::ZWaveIP => write!(f, "Z-Wave for IP"),
        }
    }
}

impl NomTryFromPrimitive for ProtocolType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown protocol type: {:#04x}", repr)
    }
}

impl Parsable for ProtocolType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ProtocolType",
            map_res(be_u8, ProtocolType::try_from_primitive),
        )(i)
    }
}

impl Serializable for ProtocolType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
