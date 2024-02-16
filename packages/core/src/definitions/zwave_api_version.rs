use crate::munch::{
    bytes::be_u8,
    combinators::{context, map, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
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

impl BytesParsable for ZWaveApiVersion {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("ZWaveApiVersion", map(be_u8(), Self::from)).parse(i)
    }
}

impl Serializable for ZWaveApiVersion {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8((*self).into())
    }
}
