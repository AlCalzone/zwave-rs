use crate::encoding::{self, BitParsable, BitSerializable, Parsable, Serializable, WriteLastNBits};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use encoding::{EncodingError, EncodingResult};
use nom::{
    bits::complete::take as take_bits, combinator::map, error::context, number::complete::be_u8,
};
use ux::u3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolDataRate {
    #[debug(format = "Z-Wave, {:?}", _0)]
    ZWave(DataRate),
    #[debug(format = "Z-Wave Long Range, 100 kbps")]
    ZWaveLongRange,
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
    #[debug(format = "9.6 kbps")]
    DataRate_9k6 = 0x01,
    #[debug(format = "40 kbps")]
    DataRate_40k = 0x02,
    #[debug(format = "100 kbps")]
    DataRate_100k = 0x03,
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
