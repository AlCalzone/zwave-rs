use crate::munch::{
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
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

impl BytesParsable for ProtocolType {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("ProtocolType", map_res(be_u8, ProtocolType::try_from)).parse(i)
    }
}

impl Serializable for ProtocolType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
