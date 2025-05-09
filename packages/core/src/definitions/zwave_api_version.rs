use crate::serialize::{self, Serializable};
use crate::parse::{
    bytes::be_u8,
    combinators::{context, map},
};
use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZWaveApiVersion {
    Official(u8),
    Legacy(u8),
}

impl Display for ZWaveApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Official(v) => write!(f, "{} (official)", v),
            Self::Legacy(v) => write!(f, "{} (legacy)", v),
        }
    }
}

impl From<u8> for ZWaveApiVersion {
    fn from(version: u8) -> Self {
        if version < 10 {
            Self::Legacy(version)
        } else {
            Self::Official(version - 9)
        }
    }
}

impl From<ZWaveApiVersion> for u8 {
    fn from(val: ZWaveApiVersion) -> Self {
        match val {
            ZWaveApiVersion::Official(v) => v + 9,
            ZWaveApiVersion::Legacy(v) => v,
        }
    }
}

impl Parsable for ZWaveApiVersion {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        context("ZWaveApiVersion", map(be_u8, Self::from)).parse(i)
    }
}

impl Serializable for ZWaveApiVersion {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        be_u8((*self).into()).serialize(output)
    }
}
