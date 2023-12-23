use std::fmt::Display;

use derive_try_from_primitive::TryFromPrimitive;
use nom::{bits::complete::take as take_bits, combinator::{map_res, map}, error::context};

use crate::encoding::{self, BitParsable, NomTryFromPrimitive};

#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Beam {
    Beam250ms = 1,
    Beam1000ms = 2,
}

impl Display for Beam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Beam::Beam250ms => write!(f, "250 ms"),
            Beam::Beam1000ms => write!(f, "1000 ms"),
        }
    }
}

impl NomTryFromPrimitive for Beam {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!(
            "Unknown binary representation for beam frequency: {:#04x}",
            repr
        )
    }
}

impl BitParsable for Beam {
    fn parse(i: encoding::BitInput) -> encoding::BitParseResult<Self> {
        context(
            "Beam",
            map_res(take_bits(2usize), |x: u8| Beam::try_from_primitive(x)),
        )(i)
    }
}

impl Beam {
    pub fn parse_opt(i: encoding::BitInput) -> encoding::BitParseResult<Option<Self>> {
        context(
            "Beam",
            map(take_bits(2usize), |x: u8| match Beam::try_from_primitive(x) {
                Ok(beam) => Some(beam),
                Err(_) => None,
            }),
        )(i)
    }
}
