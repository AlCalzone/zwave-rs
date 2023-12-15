use std::fmt::Display;

use crate::encoding::{self, NomTryFromPrimitive, Parsable, Serializable};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use nom::{combinator::map_res, error::context, number::complete::be_u8};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
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

impl NomTryFromPrimitive for RfRegion {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown RF region: {:#04x}", repr)
    }
}

impl Parsable for RfRegion {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context("RfRegion", map_res(be_u8, RfRegion::try_from_primitive))(i)
    }
}

impl Serializable for RfRegion {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
