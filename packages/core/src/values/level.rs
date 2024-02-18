use crate::serialize::{self, Serializable};
use crate::parse::{bytes::be_u8, combinators::map_res};
use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use std::fmt::Display;

pub const LEVEL_MAX: u8 = 99;
pub const LEVEL_UNKNOWN: u8 = 0xfe;
pub const LEVEL_ON: u8 = 0xff;

/// Represents a value of type Level (0-99) that is reported a device
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelReport {
    Level(u8),
    Unknown,
}

impl TryFrom<u8> for LevelReport {
    type Error = TryFromReprError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            LEVEL_UNKNOWN => Ok(Self::Unknown),
            LEVEL_ON => Ok(Self::Level(LEVEL_MAX)),
            0..=LEVEL_MAX => Ok(Self::Level(value)),
            _ => Err(TryFromReprError::Invalid(value)),
        }
    }
}

impl Display for LevelReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelReport::Level(level) => write!(f, "{}", level),
            LevelReport::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Parsable for LevelReport {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Serializable for LevelReport {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        let val = match self {
            Self::Level(level) => *level,
            Self::Unknown => LEVEL_UNKNOWN,
        };
        be_u8(val).serialize(output)
    }
}

/// Represents a value of type Level (0-99, 255) that is sent to a device
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelSet {
    /// Convenience value for setting the level to 0. Level(0) is the canonical representation.
    Off,
    Level(u8),
    On,
}

impl Canonical for LevelSet {
    fn to_canonical(&self) -> Self {
        match self {
            // Level 0 is canonical
            Self::Off => Self::Level(0),
            // Allowed levels are 0-99
            Self::Level(level) if level <= &99u8 => Self::Level(*level),
            // Translate level 255 to On
            Self::Level(LEVEL_ON) => Self::On,
            // Limit all other values to 99
            Self::Level(_) => Self::Level(LEVEL_MAX),
            // On is canonical
            Self::On => Self::On,
        }
    }
}

impl TryFrom<u8> for LevelSet {
    type Error = TryFromReprError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Level(0)),
            LEVEL_ON => Ok(Self::On),
            1..=LEVEL_MAX => Ok(Self::Level(value)),
            _ => Err(TryFromReprError::Invalid(value)),
        }
    }
}

impl Display for LevelSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelSet::Off => write!(f, "Off"),
            LevelSet::Level(level) => write!(f, "{}", level),
            LevelSet::On => write!(f, "Turn on"),
        }
    }
}

impl Parsable for LevelSet {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Serializable for LevelSet {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        let val = match self.to_canonical() {
            Self::Off => 0,
            Self::Level(level) => level,
            Self::On => LEVEL_ON,
        };
        be_u8(val).serialize(output)
    }
}

#[cfg(test)]
mod test {
    use super::{LevelReport, LevelSet};
    use crate::prelude::*;
    use std::convert::TryFrom;

    #[test]
    fn test_level_report() {
        assert_eq!(LevelReport::try_from(0), Ok(LevelReport::Level(0)));
        assert_eq!(LevelReport::try_from(1), Ok(LevelReport::Level(1)));
        assert_eq!(LevelReport::try_from(99), Ok(LevelReport::Level(99)));
        assert_eq!(
            LevelReport::try_from(100),
            Err(TryFromReprError::Invalid(100))
        );
        assert_eq!(LevelReport::try_from(0xfe), Ok(LevelReport::Unknown));
        assert_eq!(LevelReport::try_from(0xff), Ok(LevelReport::Level(99)));
    }

    #[test]
    fn test_level_set() {
        assert_eq!(LevelSet::try_from(0), Ok(LevelSet::Level(0)));
        assert_eq!(LevelSet::try_from(1), Ok(LevelSet::Level(1)));
        assert_eq!(LevelSet::try_from(99), Ok(LevelSet::Level(99)));
        assert_eq!(LevelSet::try_from(100), Err(TryFromReprError::Invalid(100)));
        assert_eq!(
            LevelSet::try_from(0xfe),
            Err(TryFromReprError::Invalid(0xfe))
        );
        assert_eq!(LevelSet::try_from(0xff), Ok(LevelSet::On));
    }
}
