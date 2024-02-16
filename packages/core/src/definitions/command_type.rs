use crate::bake::Encoder;
use crate::prelude::*;
use crate::{
    bake,
    munch::{
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

impl BytesParsable for CommandType {
    fn parse(i: &mut Bytes) -> munch::ParseResult<Self> {
        context("CommandType", map_res(be_u8(), CommandType::try_from)).parse(i)
    }
}

impl Encoder for CommandType {
    fn write(&self, output: &mut bytes::BytesMut) {
        use bake::bytes::be_u8;
        be_u8(*self as u8).write(output);
    }
}
