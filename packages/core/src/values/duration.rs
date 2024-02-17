use crate::bake::{self, Encoder};
use crate::munch::{bytes::be_u8, combinators::map_res};
use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use num_traits::clamp;

const MINUTES_MASK: u8 = 0b1000_0000;
const SECONDS_MASK: u8 = 0b0111_1111;

#[derive(Default, Debug, Clone, Copy)]
pub enum DurationSet {
    Seconds(u8),
    Minutes(u8),
    #[default]
    Default,
}

impl TryFrom<u8> for DurationSet {
    type Error = TryFromReprError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xff => Ok(Self::Default),
            0..=SECONDS_MASK => Ok(Self::Seconds(value)),
            _ => Ok(Self::Minutes((value & SECONDS_MASK) + 1)),
        }
    }
}

impl Parsable for DurationSet {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Encoder for DurationSet {
    fn write(&self, output: &mut BytesMut) {
        use bake::bytes::be_u8;
        be_u8((*self).into()).write(output)
    }
}

impl From<DurationSet> for u8 {
    fn from(value: DurationSet) -> Self {
        match value.to_canonical() {
            DurationSet::Seconds(seconds) => seconds & SECONDS_MASK,
            DurationSet::Minutes(minutes) => MINUTES_MASK | ((minutes - 1) & SECONDS_MASK),
            DurationSet::Default => 0xff,
        }
    }
}

impl Canonical for DurationSet {
    fn to_canonical(&self) -> Self {
        // Durations for a set command can represent 0..127 seconds or 1..127 minutes
        match self {
            Self::Default => Self::Default,
            Self::Minutes(m) => Self::Minutes(clamp(*m, 1, 127)),
            Self::Seconds(s) if s <= &127u8 => *self,
            // Round seconds > 127 to minutes
            Self::Seconds(s) => {
                let minutes = (*s as f32 / 60.0).round() as u8;
                Self::Minutes(clamp(minutes, 1, 127))
            }
        }
    }
}

impl PartialEq for DurationSet {
    fn eq(&self, other: &Self) -> bool {
        match (self.to_canonical(), other.to_canonical()) {
            (Self::Seconds(l0), Self::Seconds(r0)) => l0 == r0,
            (Self::Minutes(l0), Self::Minutes(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum DurationReport {
    Seconds(u8),
    Minutes(u8),
    #[default] // By default, we don't know the duration for a report
    Unknown,
}

impl TryFrom<u8> for DurationReport {
    type Error = TryFromReprError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xfe => Ok(Self::Unknown),
            0xff => Err(TryFromReprError::Invalid(value)), // reserved value
            0..=SECONDS_MASK => Ok(Self::Seconds(value)),
            _ => Ok(Self::Minutes((value & SECONDS_MASK) + 1)),
        }
    }
}

impl Parsable for DurationReport {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Encoder for DurationReport {
    fn write(&self, output: &mut BytesMut) {
        use bake::bytes::be_u8;
        be_u8((*self).into()).write(output)
    }
}

impl From<DurationReport> for u8 {
    fn from(value: DurationReport) -> Self {
        match value.to_canonical() {
            DurationReport::Seconds(seconds) => seconds & SECONDS_MASK,
            DurationReport::Minutes(minutes) => MINUTES_MASK | ((minutes - 1) & SECONDS_MASK),
            DurationReport::Unknown => 0xfe,
        }
    }
}

impl Canonical for DurationReport {
    fn to_canonical(&self) -> Self {
        // Durations for a report command can represent 0..127 seconds or 1..126 minutes
        match self {
            Self::Unknown => Self::Unknown,
            Self::Minutes(m) => Self::Minutes(clamp(*m, 1, 126)),
            Self::Seconds(s) if s <= &127u8 => *self,
            // Round seconds > 127 to minutes
            Self::Seconds(s) => {
                let minutes = (*s as f32 / 60.0).round() as u8;
                Self::Minutes(clamp(minutes, 1, 126))
            }
        }
    }
}

impl PartialEq for DurationReport {
    fn eq(&self, other: &Self) -> bool {
        match (self.to_canonical(), other.to_canonical()) {
            (Self::Seconds(l0), Self::Seconds(r0)) => l0 == r0,
            (Self::Minutes(l0), Self::Minutes(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::convert::TryFrom;

    #[test]
    fn test_duration_report() {
        assert_eq!(DurationReport::try_from(0), Ok(DurationReport::Seconds(0)));
        assert_eq!(DurationReport::try_from(1), Ok(DurationReport::Seconds(1)));
        assert_eq!(
            DurationReport::try_from(127),
            Ok(DurationReport::Seconds(127))
        );
        assert_eq!(u8::from(DurationReport::Seconds(0)), 0u8);
        assert_eq!(u8::from(DurationReport::Seconds(1)), 1u8);
        assert_eq!(u8::from(DurationReport::Seconds(127)), 127u8);

        assert_eq!(
            DurationReport::try_from(128),
            Ok(DurationReport::Minutes(1))
        );
        assert_eq!(
            DurationReport::try_from(129),
            Ok(DurationReport::Minutes(2))
        );
        assert_eq!(
            DurationReport::try_from(253),
            Ok(DurationReport::Minutes(126))
        );
        assert_eq!(u8::from(DurationReport::Minutes(1)), 128u8);
        assert_eq!(u8::from(DurationReport::Minutes(2)), 129u8);
        assert_eq!(u8::from(DurationReport::Minutes(126)), 253u8);
        // Conversion to u8 normalizes
        assert_eq!(u8::from(DurationReport::Minutes(127)), 253u8);

        assert_eq!(DurationReport::try_from(0xfe), Ok(DurationReport::Unknown));
        assert_eq!(u8::from(DurationReport::Unknown), 0xfeu8);

        assert_eq!(
            DurationReport::try_from(0xff),
            Err(TryFromReprError::Invalid(0xff))
        );

        assert_eq!(
            DurationReport::Seconds(128).to_canonical(),
            DurationReport::Minutes(2)
        );
        assert_eq!(
            DurationReport::Minutes(127).to_canonical(),
            DurationReport::Minutes(126)
        );
    }

    #[test]
    fn test_duration_set() {
        assert_eq!(DurationSet::try_from(0), Ok(DurationSet::Seconds(0)));
        assert_eq!(DurationSet::try_from(1), Ok(DurationSet::Seconds(1)));
        assert_eq!(DurationSet::try_from(127), Ok(DurationSet::Seconds(127)));
        assert_eq!(u8::from(DurationSet::Seconds(0)), 0u8);
        assert_eq!(u8::from(DurationSet::Seconds(1)), 1u8);
        assert_eq!(u8::from(DurationSet::Seconds(127)), 127u8);
        // Conversion to u8 normalizes
        assert_eq!(u8::from(DurationSet::Seconds(128)), 129u8); // 2 minutes

        assert_eq!(DurationSet::try_from(128), Ok(DurationSet::Minutes(1)));
        assert_eq!(DurationSet::try_from(129), Ok(DurationSet::Minutes(2)));
        assert_eq!(DurationSet::try_from(254), Ok(DurationSet::Minutes(127)));
        assert_eq!(u8::from(DurationSet::Minutes(1)), 128u8);
        assert_eq!(u8::from(DurationSet::Minutes(2)), 129u8);
        assert_eq!(u8::from(DurationSet::Minutes(127)), 254u8);
        // Conversion to u8 normalizes
        assert_eq!(u8::from(DurationSet::Minutes(128)), 254u8);

        assert_eq!(DurationSet::try_from(0xff), Ok(DurationSet::Default));
        assert_eq!(u8::from(DurationSet::Default), 0xffu8);

        assert_eq!(
            DurationSet::Seconds(128).to_canonical(),
            DurationSet::Minutes(2)
        );
        assert_eq!(
            DurationSet::Minutes(128).to_canonical(),
            DurationSet::Minutes(127)
        );
    }
}
