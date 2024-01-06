use crate::encoding;
use crate::prelude::*;
use cookie_factory as cf;
use nom::{combinator::map_res, error::context, number::complete::be_u8};
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum RoutingScheme {
    Idle,
    Direct,
    Priority,
    LWR,
    NLWR,
    Auto,
    ResortDirect,
    Explore,
}

impl Display for RoutingScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingScheme::Idle => write!(f, "Idle"),
            RoutingScheme::Direct => write!(f, "Direct"),
            RoutingScheme::Priority => write!(f, "Priority route"),
            RoutingScheme::LWR => write!(f, "LWR"),
            RoutingScheme::NLWR => write!(f, "NLWR"),
            RoutingScheme::Auto => write!(f, "Auto route"),
            RoutingScheme::ResortDirect => write!(f, "Resort to direct"),
            RoutingScheme::Explore => write!(f, "Explorer Frame"),
        }
    }
}

impl NomTryFromPrimitive for RoutingScheme {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown routing scheme: {:#04x}", repr)
    }
}

impl Parsable for RoutingScheme {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "RoutingScheme",
            map_res(be_u8, RoutingScheme::try_from_primitive),
        )(i)
    }
}

impl Serializable for RoutingScheme {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
