use crate::encoding::{self, NomTryFromPrimitive};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use encoding::{Parsable, Serializable};
use nom::{combinator::map_res, number::complete::be_u8};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum BasicDeviceType {
    PortableController = 0x01,
    StaticController = 0x02,
    EndNode = 0x03,
    RoutingEndNode = 0x04,
}

impl NomTryFromPrimitive for BasicDeviceType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown basic device type: {:#04x}", repr)
    }
}

impl Display for BasicDeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BasicDeviceType::PortableController => write!(f, "Portable Controller"),
            BasicDeviceType::StaticController => write!(f, "Static Controller"),
            BasicDeviceType::EndNode => write!(f, "End Node"),
            BasicDeviceType::RoutingEndNode => write!(f, "Routing End Node"),
        }
    }
}

impl Parsable for BasicDeviceType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        map_res(be_u8, BasicDeviceType::try_from_primitive)(i)
    }
}

impl Serializable for BasicDeviceType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8((*self) as u8)
    }
}
