use crate::encoding::{self, Parsable, Serializable};

use std::fmt::Display;
use cookie_factory as cf;
use nom::{combinator::map, error::context, number::complete::be_u16};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ChipType {
    ZW0102 = 0x0102,
    ZW0201 = 0x0201,
    ZW0301 = 0x0301,
    ZM040x = 0x0401,
    ZW050x = 0x0501,
    EFR32xG1x = 0x0700,
    EFR32xG2x = 0x0800,
    Unknown(u16),
}

impl Display for ChipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipType::ZW0102 => write!(f, "ZW0102"),
            ChipType::ZW0201 => write!(f, "ZW0201"),
            ChipType::ZW0301 => write!(f, "ZW0301"),
            ChipType::ZM040x => write!(f, "ZM0401 / ZM4102 / SD3402"),
            ChipType::ZW050x => write!(f, "ZW050x"),
            ChipType::EFR32xG1x => write!(f, "EFR32ZG14 / ZGM130S"),
            ChipType::EFR32xG2x => write!(f, "EFR32ZG23 / ZGM230S"),
            ChipType::Unknown(v) => write!(f, "Unknown({:#04x})", v),
        }
    }
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
