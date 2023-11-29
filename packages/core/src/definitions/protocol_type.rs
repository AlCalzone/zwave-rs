use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use nom::{combinator::map, error::context, number::complete::be_u8};
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

impl Parsable for ProtocolType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ProtocolType",
            map(be_u8, |x| ProtocolType::try_from(x).unwrap()),
        )(i)
    }
}

impl Serializable for ProtocolType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
