use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum RfRegion {
    #[debug(format = "Europe")]
    EU = 0,
    #[debug(format = "USA")]
    US = 1,
    #[debug(format = "Australia / New Zealand")]
    ANZ = 2,
    #[debug(format = "Hong Kong")]
    HK = 3,
    #[debug(format = "India")]
    IN = 5,
    #[debug(format = "Israel")]
    IL = 6,
    #[debug(format = "Russia")]
    RU = 7,
    #[debug(format = "China")]
    CN = 8,
    #[debug(format = "USA (Long Range)")]
    US_LongRange = 9,
    #[debug(format = "Japan")]
    JP = 32,
    #[debug(format = "Korea")]
    KR = 33,
    #[debug(format = "Unknown")]
    Unknown = 254,
    #[debug(format = "Default (Europe)")]
    Default = 255,
    
}

impl Parsable for RfRegion {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "RfRegion",
            map(be_u8, |x| RfRegion::try_from(x).unwrap()),
        )(i)
    }
}

impl Serializable for RfRegion {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
