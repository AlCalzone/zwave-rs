use crate::encoding::{self};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum CommandType {
    Request = 0x00,
    Response = 0x01,
}

impl CommandType {
    pub fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "CommandType",
            map(be_u8, |x| CommandType::try_from(x).unwrap()),
        )(i)
    }

    pub fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(*self as u8)
    }
}
