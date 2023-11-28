use crate::encoding::{self, BitParsable, BitSerializable, Parsable, Serializable, WriteLastNBits};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{
    bits::complete::take as take_bits,
    combinator::map,
    error::context,
    number::complete::{be_u16, be_u8},
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ProtocolType {
    #[debug(format = "Z-Wave")]
    ZWave,
    #[debug(format = "Z-Wave AV")]
    ZWaveAV,
    #[debug(format = "Z-Wave for IP")]
    ZWaveIP,
}

impl Parsable for ProtocolType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ProtocolType",
            map(be_u8, |x| ProtocolType::try_from(x).unwrap()),
        )(i)
    }
}

impl Serializable for ProtocolType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
