use crate::encoding::{self, NomTryFromPrimitive, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map_res, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum TransmitStatus {
    Ok = 0x00,
    NoAck = 0x01,
    Fail = 0x02,
    NotIdle = 0x03,
    NoRoute = 0x04,
}

impl NomTryFromPrimitive for TransmitStatus {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown transmit status: {:#04x}", repr)
    }
}

impl Parsable for TransmitStatus {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "TransmitStatus",
            map_res(be_u8, TransmitStatus::try_from_primitive),
        )(i)
    }
}

impl Serializable for TransmitStatus {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
