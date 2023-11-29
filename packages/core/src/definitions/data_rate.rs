use crate::encoding::{self, BitParsable, BitSerializable, Parsable, Serializable, WriteLastNBits};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use encoding::{EncodingError, EncodingResult};
use nom::{
    bits::complete::take as take_bits, combinator::map, error::context, number::complete::be_u8,
};
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
    type Error = EncodingError;

    fn try_from(rate: u8) -> EncodingResult<Self> {
        match rate {
            0x01 => Ok(Self::ZWave(DataRate::DataRate_9k6)),
            0x02 => Ok(Self::ZWave(DataRate::DataRate_40k)),
            0x03 => Ok(Self::ZWave(DataRate::DataRate_100k)),
            0x04 => Ok(Self::ZWaveLongRange),
            _ => Err(EncodingError::Parse(Some(format!(
                "Invalid ProtocolDataRate: {:?}",
                rate
            )))),
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
    fn parse(i: encoding::BitInput) -> encoding::BitParseResult<Self> {
        context(
            "ProtocolDataRate",
            map(take_bits(3usize), |x: u8| {
                ProtocolDataRate::try_from(x).unwrap()
            }),
        )(i)
    }
}

impl Parsable for ProtocolDataRate {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ProtocolDataRate",
            map(be_u8, |x| ProtocolDataRate::try_from(x).unwrap()),
        )(i)
    }
}

impl BitSerializable for ProtocolDataRate {
    fn write(&self, b: &mut encoding::BitOutput) {
        b.write_last_n_bits(u8::from(*self), 3);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
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

impl Parsable for DataRate {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context("DataRate", map(be_u8, |x| DataRate::try_from(x).unwrap()))(i)
    }
}

impl Serializable for DataRate {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
