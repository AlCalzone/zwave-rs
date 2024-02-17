use crate::munch::{
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::{BytesMut, Bytes};
use crate::bake::{self, Encoder};
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

impl Parsable for RoutingScheme {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("RoutingScheme", map_res(be_u8, RoutingScheme::try_from)).parse(i)
    }
}

impl Encoder for RoutingScheme {
    fn write(&self, output: &mut BytesMut) {
        use bake::bytes::be_u8;
        be_u8(*self as u8).write(output)
    }
}
