use crate::prelude::*;
use crate::{
    serialize,
    parse::{
        self,
        bytes::be_u8,
        combinators::{context, map_res},
    },
};
use bytes::Bytes;
use proc_macros::TryFromRepr;

use custom_debug_derive::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum CommandType {
    Request = 0x00,
    Response = 0x01,
}

impl Parsable for CommandType {
    fn parse(i: &mut Bytes) -> parse::ParseResult<Self> {
        context("CommandType", map_res(be_u8, CommandType::try_from)).parse(i)
    }
}

impl Serializable for CommandType {
    fn serialize(&self, output: &mut bytes::BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output);
    }
}
