use crate::parse::{
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::{BytesMut, Bytes};
use crate::serialize::{self, Serializable};
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum RfRegion {
    EU = 0,
    US = 1,
    ANZ = 2,
    HK = 3,
    IN = 5,
    IL = 6,
    RU = 7,
    CN = 8,
    US_LongRange = 9,
    JP = 32,
    KR = 33,
    Unknown = 254,
    #[default]
    Default = 255,
}

impl Display for RfRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RfRegion::EU => write!(f, "Europe"),
            RfRegion::US => write!(f, "USA"),
            RfRegion::ANZ => write!(f, "Australia / New Zealand"),
            RfRegion::HK => write!(f, "Hong Kong"),
            RfRegion::IN => write!(f, "India"),
            RfRegion::IL => write!(f, "Israel"),
            RfRegion::RU => write!(f, "Russia"),
            RfRegion::CN => write!(f, "China"),
            RfRegion::US_LongRange => write!(f, "USA (Long Range)"),
            RfRegion::JP => write!(f, "Japan"),
            RfRegion::KR => write!(f, "Korea"),
            RfRegion::Unknown => write!(f, "Unknown"),
            RfRegion::Default => write!(f, "Default (Europe)"),
        }
    }
}

impl Parsable for RfRegion {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        context("RfRegion", map_res(be_u8, Self::try_from)).parse(i)
    }
}

impl Serializable for RfRegion {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output)
    }
}
