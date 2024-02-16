use crate::encoding;
use crate::encoding::WriteLastNBits;
use crate::munch::{
    bits,
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolDataRate {
    ZWave(DataRate),
    ZWaveLongRange,
}

impl Display for ProtocolDataRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolDataRate::ZWave(rate) => write!(f, "Z-Wave, {}", rate),
            ProtocolDataRate::ZWaveLongRange => write!(f, "Z-Wave Long Range, 100 kbit/s"),
        }
    }
}

impl TryFrom<u8> for ProtocolDataRate {
    type Error = TryFromReprError<u8>;

    fn try_from(rate: u8) -> Result<Self, Self::Error> {
        match rate {
            0x01 => Ok(Self::ZWave(DataRate::DataRate_9k6)),
            0x02 => Ok(Self::ZWave(DataRate::DataRate_40k)),
            0x03 => Ok(Self::ZWave(DataRate::DataRate_100k)),
            0x04 => Ok(Self::ZWaveLongRange),
            _ => Err(TryFromReprError::Invalid(rate)),
        }
    }
}

impl From<ProtocolDataRate> for u8 {
    fn from(rate: ProtocolDataRate) -> Self {
        match rate {
            ProtocolDataRate::ZWave(rate) => rate as u8,
            ProtocolDataRate::ZWaveLongRange => 0x04,
        }
    }
}

impl BitParsable for ProtocolDataRate {
    fn parse(i: &mut (Bytes, usize)) -> crate::munch::ParseResult<Self> {
        context(
            "ProtocolDataRate",
            map_res(bits::take(3usize), |x: u8| ProtocolDataRate::try_from(x)),
        )
        .parse(i)
    }
}

impl BytesParsable for ProtocolDataRate {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context(
            "ProtocolDataRate",
            map_res(be_u8, ProtocolDataRate::try_from),
        )
        .parse(i)
    }
}

impl BitSerializable for ProtocolDataRate {
    fn write(&self, b: &mut encoding::BitOutput) {
        b.write_last_n_bits(u8::from(*self), 3);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromRepr)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum DataRate {
    DataRate_9k6 = 0x01,
    DataRate_40k = 0x02,
    DataRate_100k = 0x03,
}

impl Display for DataRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataRate::DataRate_9k6 => write!(f, "9.6 kbit/s"),
            DataRate::DataRate_40k => write!(f, "40 kbit/s"),
            DataRate::DataRate_100k => write!(f, "100 kbit/s"),
        }
    }
}

impl BytesParsable for DataRate {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("DataRate", map_res(be_u8, DataRate::try_from)).parse(i)
    }
}

impl Serializable for DataRate {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
