use crate::{
    parse::{
        bytes::be_u8,
        combinators::{context, map_res},
    },
    prelude::*,
};
use bytes::{BytesMut, Bytes};
use crate::serialize::{self, Serializable};
use proc_macros::TryFromRepr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum TransmitStatus {
    Ok = 0x00,
    NoAck = 0x01,
    Fail = 0x02,
    NotIdle = 0x03,
    NoRoute = 0x04,
}

impl Parsable for TransmitStatus {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        context("TransmitStatus", map_res(be_u8, TransmitStatus::try_from)).parse(i)
    }
}

impl Serializable for TransmitStatus {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output)
    }
}
