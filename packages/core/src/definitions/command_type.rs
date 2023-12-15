use crate::encoding::{self, NomTryFromPrimitive};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map_res, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum CommandType {
    Request = 0x00,
    Response = 0x01,
}

impl NomTryFromPrimitive for CommandType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown CommandType: {:#04x}", repr)
    }
}

impl CommandType {
    pub fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "CommandType",
            map_res(be_u8, CommandType::try_from_primitive),
        )(i)
    }

    pub fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(*self as u8)
    }
}
