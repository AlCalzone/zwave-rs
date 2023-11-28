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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ChipType {
    #[debug(format = "ZW0102")]
    ZW0102 = 0x0102,
    #[debug(format = "ZW0201")]
    ZW0201 = 0x0201,
    #[debug(format = "ZW0301")]
    ZW0301 = 0x0301,
    #[debug(format = "ZM0401 / ZM4102 / SD3402")]
    ZM040x = 0x0401,
    #[debug(format = "ZW050x")]
    ZW050x = 0x0501,
    #[debug(format = "EFR32ZG14 / ZGM130S")]
    EFR32xG1x = 0x0700,
    #[debug(format = "EFR32ZG23 / ZGM230S")]
    EFR32xG2x = 0x0800,
    Unknown(u16),
}

impl From<u16> for ChipType {
    fn from(value: u16) -> Self {
        match value {
            0x0102 => Self::ZW0102,
            0x0201 => Self::ZW0201,
            0x0301 => Self::ZW0301,
            0x0401 => Self::ZM040x,
            0x0501 => Self::ZW050x,
            0x0700 => Self::EFR32xG1x,
            0x0800 => Self::EFR32xG2x,
            _ => Self::Unknown(value),
        }
    }
}

impl Parsable for ChipType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context("ChipType", map(be_u16, ChipType::from))(i)
    }
}

impl Serializable for ChipType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u16((*self).into())
    }
}

impl From<ChipType> for u16 {
    fn from(val: ChipType) -> Self {
        match val {
            ChipType::ZW0102 => 0x0102,
            ChipType::ZW0201 => 0x0201,
            ChipType::ZW0301 => 0x0301,
            ChipType::ZM040x => 0x0401,
            ChipType::ZW050x => 0x0501,
            ChipType::EFR32xG1x => 0x0700,
            ChipType::EFR32xG2x => 0x0800,
            ChipType::Unknown(v) => v,
        }
    }
}

