use crate::bake::{self, Encoder};
use crate::munch::{bytes::be_u8, combinators::map_res};
use bytes::{Bytes, BytesMut};
use crate::prelude::*;
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum BasicDeviceType {
    PortableController = 0x01,
    StaticController = 0x02,
    EndNode = 0x03,
    RoutingEndNode = 0x04,
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
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Encoder for BasicDeviceType {
    fn write(&self, output: &mut BytesMut) {
        use bake::bytes::be_u8;
        be_u8(*self as u8).write(output)
    }
}
