use crate::encoding::Serializable;
use crate::munch::{
    bytes::be_i8,
    combinators::{context, map},
};
use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum RSSI {
    Measured(i8),
    NotAvailable = 127,
    ReceiverSaturated = 126,
    NoSignalDetected = 125,
}

impl Display for RSSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RSSI::Measured(rssi) => write!(f, "{} dBm", rssi),
            RSSI::NotAvailable => write!(f, "N/A"),
            RSSI::ReceiverSaturated => write!(f, "Receiver saturated"),
            RSSI::NoSignalDetected => write!(f, "No signal detected"),
        }
    }
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

impl BytesParsable for RSSI {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context("RSSI", map(be_i8(), RSSI::from)).parse(i)
    }
}

impl Serializable for RSSI {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_i8((*self).into())
    }
}
