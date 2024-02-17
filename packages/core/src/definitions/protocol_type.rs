use crate::munch::{
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::{BytesMut, Bytes};
use crate::bake::{self, Encoder};
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

impl Parsable for ProtocolType {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("ProtocolType", map_res(be_u8, ProtocolType::try_from)).parse(i)
    }
}

impl Encoder for ProtocolType {
    fn write(&self, output: &mut BytesMut) {
        use bake::bytes::be_u8;
        be_u8(*self as u8).write(output)
    }
}
