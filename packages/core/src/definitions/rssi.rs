use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use nom::{combinator::map, error::context, number::complete::be_i8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum RSSI {
    #[debug(fmt = "{} dBm", _0)]
    Measured(i8),
    #[debug(fmt = "N/A")]
    NotAvailable = 127,
    #[debug(fmt = "Receiver saturated")]
    ReceiverSaturated = 126,
    #[debug(fmt = "No signal detected")]
    NoSignalDetected = 125,
}

impl RSSI {
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::NotAvailable | Self::ReceiverSaturated | Self::NoSignalDetected
        )
    }
}

impl From<i8> for RSSI {
    fn from(raw: i8) -> Self {
        match raw {
            127 => Self::NotAvailable,
            126 => Self::ReceiverSaturated,
            125 => Self::NoSignalDetected,
            raw => Self::Measured(raw),
        }
    }
}

impl From<RSSI> for i8 {
    fn from(val: RSSI) -> Self {
        val.into()
    }
}

impl Parsable for RSSI {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context("RSSI", map(be_i8, RSSI::from))(i)
    }
}

impl Serializable for RSSI {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_i8((*self).into())
    }
}
