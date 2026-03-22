use crate::parse::{
    bits,
    combinators::{context, map, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use proc_macros::TryFromRepr;
use core::fmt::Display;

#[derive(Debug, Clone, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum Beam {
    Beam250ms = 1,
    Beam1000ms = 2,
}

impl Display for Beam {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Beam::Beam250ms => write!(f, "250 ms"),
            Beam::Beam1000ms => write!(f, "1000 ms"),
        }
    }
}

impl BitParsable for Beam {
    fn parse(i: &mut (Bytes, usize)) -> crate::parse::ParseResult<Self> {
        context(
            "Beam",
            map_res(bits::take(2usize), |x: u8| Beam::try_from(x)),
        )
        .parse(i)
    }
}

impl Beam {
    pub fn parse_opt(i: &mut (Bytes, usize)) -> crate::parse::ParseResult<Option<Self>> {
        context(
            "Beam",
            map(bits::take(2usize), |x: u8| Beam::try_from(x).ok()),
        )
        .parse(i)
    }
}
