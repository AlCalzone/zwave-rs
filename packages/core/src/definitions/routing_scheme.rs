use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum RoutingScheme {
    #[debug(fmt = "Idle")]
    Idle,
    #[debug(fmt = "Direct")]
    Direct,
    #[debug(fmt = "Priority route")]
    Priority,
    #[debug(fmt = "LWR")]
    LWR,
    #[debug(fmt = "NLWR")]
    NLWR,
    #[debug(fmt = "Auto route")]
    Auto,
    #[debug(fmt = "Resort to direct")]
    ResortDirect,
    #[debug(fmt = "Explorer Frame")]
    Explore,
}

impl Parsable for RoutingScheme {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "RoutingScheme",
            map(be_u8, |x| RoutingScheme::try_from(x).unwrap()),
        )(i)
    }
}

impl Serializable for RoutingScheme {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
