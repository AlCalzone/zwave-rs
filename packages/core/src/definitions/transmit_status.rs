use crate::{
    munch::{
        bytes::be_u8,
        combinators::{context, map_res},
    },
    prelude::*,
};
use bytes::Bytes;
use cookie_factory as cf;
use custom_debug_derive::Debug;
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

impl BytesParsable for TransmitStatus {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("TransmitStatus", map_res(be_u8, TransmitStatus::try_from)).parse(i)
    }
}

impl Serializable for TransmitStatus {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
