use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;

use nom::{combinator::map, error::context, number::complete::be_u8};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZWaveApiVersion {
    Official(u8),
    Legacy(u8),
}

impl fmt::Debug for ZWaveApiVersion {
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
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context("ZWaveApiVersion", map(be_u8, ZWaveApiVersion::from))(i)
    }
}

impl Serializable for ZWaveApiVersion {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8((*self).into())
    }
}
