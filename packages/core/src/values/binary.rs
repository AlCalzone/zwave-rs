use crate::munch::{bytes::be_u8, combinators::map_res};
use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;

// All values from 1 to BINARY_SET_MAX are interpreted as ON in SET commands
pub const BINARY_SET_MAX: u8 = 99;
pub const BINARY_UNKNOWN: u8 = 0xfe;
pub const BINARY_ON: u8 = 0xff;

/// Represents a value of type Binary (0-99) that is reported a device
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BinaryReport {
    Off = 0,
    Unknown = BINARY_UNKNOWN,
    On = BINARY_ON,
}

impl TryFrom<u8> for BinaryReport {
    type Error = TryFromReprError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            BINARY_UNKNOWN => Ok(Self::Unknown),
            BINARY_ON => Ok(Self::On),
            _ => Err(TryFromReprError::Invalid(value)),
        }
    }
}

impl BytesParsable for BinaryReport {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Serializable for BinaryReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(*self as u8)
    }
}

impl From<Option<bool>> for BinaryReport {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::On,
            Some(false) => Self::Off,
            None => Self::Unknown,
        }
    }
}

impl From<BinaryReport> for Option<bool> {
    fn from(value: BinaryReport) -> Self {
        match value {
            BinaryReport::On => Some(true),
            BinaryReport::Off => Some(false),
            BinaryReport::Unknown => None,
        }
    }
}

impl From<bool> for BinaryReport {
    fn from(value: bool) -> Self {
        if value {
            Self::On
        } else {
            Self::Off
        }
    }
}

impl TryFrom<BinaryReport> for bool {
    type Error = BinaryReport;

    fn try_from(value: BinaryReport) -> Result<Self, Self::Error> {
        match value {
            BinaryReport::On => Ok(true),
            BinaryReport::Off => Ok(false),
            BinaryReport::Unknown => Err(value),
        }
    }
}

impl From<LevelReport> for BinaryReport {
    fn from(value: LevelReport) -> Self {
        match value {
            LevelReport::Level(0) => Self::Off,
            LevelReport::Level(_) => Self::On,
            LevelReport::Unknown => Self::Unknown,
        }
    }
}

impl From<BinaryReport> for LevelReport {
    fn from(value: BinaryReport) -> Self {
        match value {
            BinaryReport::On => Self::Level(LEVEL_MAX),
            BinaryReport::Off => Self::Level(0),
            BinaryReport::Unknown => Self::Unknown,
        }
    }
}

/// Represents a value of type Binary (On/Off) that is sent to a device
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BinarySet {
    Off = 0,
    On = BINARY_ON,
}

impl TryFrom<u8> for BinarySet {
    type Error = TryFromReprError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1..=BINARY_SET_MAX | BINARY_ON => Ok(Self::On),
            _ => Err(TryFromReprError::Invalid(value)),
        }
    }
}

impl BytesParsable for BinarySet {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        map_res(be_u8, Self::try_from).parse(i)
    }
}

impl Serializable for BinarySet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(*self as u8)
    }
}

impl From<bool> for BinarySet {
    fn from(value: bool) -> Self {
        if value {
            Self::On
        } else {
            Self::Off
        }
    }
}

impl From<BinarySet> for bool {
    fn from(value: BinarySet) -> Self {
        match value {
            BinarySet::On => true,
            BinarySet::Off => false,
        }
    }
}

impl From<LevelSet> for BinarySet {
    fn from(value: LevelSet) -> Self {
        match value {
            LevelSet::Off | LevelSet::Level(0) => Self::Off,
            LevelSet::Level(_) => Self::On,
            LevelSet::On => Self::On,
        }
    }
}

impl From<BinarySet> for LevelSet {
    fn from(value: BinarySet) -> Self {
        match value {
            BinarySet::On => Self::On,
            BinarySet::Off => Self::Off,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::values::LevelSet;
    use crate::prelude::*;

    #[test]
    fn test_binary_report() {
        use super::BinaryReport;
        use std::convert::TryFrom;

        assert_eq!(BinaryReport::try_from(0), Ok(BinaryReport::Off));
        assert_eq!(BinaryReport::try_from(1), Err(TryFromReprError::Invalid(1)));
        assert_eq!(BinaryReport::try_from(99), Err(TryFromReprError::Invalid(99)));
        assert_eq!(BinaryReport::try_from(100), Err(TryFromReprError::Invalid(100)));
        assert_eq!(BinaryReport::try_from(0xfe), Ok(BinaryReport::Unknown));
        assert_eq!(BinaryReport::try_from(0xff), Ok(BinaryReport::On));
    }

    #[test]
    fn test_binary_set() {
        use super::BinarySet;
        use std::convert::TryFrom;

        assert_eq!(BinarySet::try_from(0), Ok(BinarySet::Off));
        assert_eq!(BinarySet::try_from(1), Ok(BinarySet::On));
        assert_eq!(BinarySet::try_from(99), Ok(BinarySet::On));
        assert_eq!(BinarySet::try_from(100), Err(TryFromReprError::Invalid(100)));
        assert_eq!(BinarySet::try_from(0xfe), Err(TryFromReprError::Invalid(0xfe)));
        assert_eq!(BinarySet::try_from(0xff), Ok(BinarySet::On));
    }

    #[test]
    fn test_binary_bool_conversion() {
        use super::{BinaryReport, BinarySet};
        use std::convert::TryFrom;

        assert_eq!(BinaryReport::from(true), BinaryReport::On);
        assert_eq!(BinaryReport::from(false), BinaryReport::Off);
        assert_eq!(BinaryReport::from(None), BinaryReport::Unknown);
        assert_eq!(BinaryReport::from(Some(true)), BinaryReport::On);
        assert_eq!(BinaryReport::from(Some(false)), BinaryReport::Off);

        assert_eq!(BinarySet::from(true), BinarySet::On);
        assert_eq!(BinarySet::from(false), BinarySet::Off);

        assert_eq!(bool::try_from(BinaryReport::On), Ok(true));
        assert_eq!(bool::try_from(BinaryReport::Off), Ok(false));
        assert_eq!(
            bool::try_from(BinaryReport::Unknown),
            Err(BinaryReport::Unknown)
        );

        assert!(bool::from(BinarySet::On));
        assert!(!bool::from(BinarySet::Off));
    }

    #[test]
    fn test_binary_level_conversion() {
        use super::{BinaryReport, BinarySet, LevelReport};

        assert_eq!(LevelReport::from(BinaryReport::On), LevelReport::Level(99));
        assert_eq!(LevelReport::from(BinaryReport::Off), LevelReport::Level(0));
        assert_eq!(
            LevelReport::from(BinaryReport::Unknown),
            LevelReport::Unknown
        );

        assert_eq!(BinaryReport::from(LevelReport::Level(0)), BinaryReport::Off);
        assert_eq!(BinaryReport::from(LevelReport::Level(1)), BinaryReport::On);
        assert_eq!(BinaryReport::from(LevelReport::Level(99)), BinaryReport::On);
        assert_eq!(
            BinaryReport::from(LevelReport::Unknown),
            BinaryReport::Unknown
        );

        assert_eq!(BinarySet::from(LevelSet::Off), BinarySet::Off);
        assert_eq!(BinarySet::from(LevelSet::On), BinarySet::On);
        assert_eq!(BinarySet::from(LevelSet::Level(0)), BinarySet::Off);
        assert_eq!(BinarySet::from(LevelSet::Level(1)), BinarySet::On);
        assert_eq!(BinarySet::from(LevelSet::Level(99)), BinarySet::On);

        assert_eq!(LevelSet::from(BinarySet::Off), LevelSet::Off);
        assert_eq!(LevelSet::from(BinarySet::On), LevelSet::On);
    }
}
