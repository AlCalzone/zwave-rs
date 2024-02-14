use crate::prelude::{Command, CommandEncodingContext};
use bytes::{Buf, Bytes, BytesMut};
use proc_macros::TryFromRepr;
use std::fmt::Display;
use zwave_core::bake::{self, Encoder};
use zwave_core::munch;
use zwave_core::prelude::*;

#[derive(Debug, TryFromRepr)]
#[repr(u8)]
pub enum SerialControlByte {
    SOF = 0x01,
    ACK = 0x06,
    NAK = 0x15,
    CAN = 0x18,
}

pub const SOF_BYTE: u8 = SerialControlByte::SOF as u8;
pub const ACK_BYTE: u8 = SerialControlByte::ACK as u8;
pub const NAK_BYTE: u8 = SerialControlByte::NAK as u8;
pub const CAN_BYTE: u8 = SerialControlByte::CAN as u8;

/// Control-flow commands
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ControlFlow {
    ACK = SerialControlByte::ACK as u8,
    NAK = SerialControlByte::NAK as u8,
    CAN = SerialControlByte::CAN as u8,
}

impl Display for ControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFlow::ACK => write!(f, "ACK"),
            ControlFlow::NAK => write!(f, "NAK"),
            ControlFlow::CAN => write!(f, "CAN"),
        }
    }
}

/// A raw serial frame, as received from the serial port
#[derive(Clone, Debug, PartialEq)]
pub enum RawSerialFrame {
    ControlFlow(ControlFlow),
    Data(Bytes),
    Garbage(Bytes),
}

/// A parsed serial frame that contains a control-flow byte or a Serial API command
#[derive(Clone, Debug, PartialEq)]
pub enum SerialFrame {
    ControlFlow(ControlFlow),
    Command(Command),
    Raw(Bytes),
}

impl RawSerialFrame {
    pub fn parse_mut(i: &mut BytesMut) -> munch::ParseResult<Self> {
        if i.remaining() == 0 {
            return Err(munch::ParseError::needed(1));
        }

        // A serial frame is either a control byte, data starting with SOF, or skipped garbage
        match i[0] {
            ACK_BYTE => {
                i.advance(1);
                Ok(Self::ControlFlow(ControlFlow::ACK))
            }
            NAK_BYTE => {
                i.advance(1);
                Ok(Self::ControlFlow(ControlFlow::NAK))
            }
            CAN_BYTE => {
                i.advance(1);
                Ok(Self::ControlFlow(ControlFlow::CAN))
            }
            SOF_BYTE => {
                // Ensure we have at least 5 bytes
                if i.len() < 5 {
                    return Err(munch::ParseError::needed(5 - i.len()));
                }
                let len = i[1] as usize;
                if i.len() < len + 2 {
                    return Err(munch::ParseError::needed(len + 2 - i.len()));
                }

                let data = i.split_to(len + 2);
                Ok(Self::Data(data.freeze()))
            }
            _ => {
                // Garbage - find the first non-garbage byte and return everything up to it
                let end_pos = i
                    .iter()
                    .position(|v| SerialControlByte::try_from(*v).is_ok());
                let garbage = match end_pos {
                    // We need at least one byte that matches the predicate
                    Some(pos) => i.split_to(pos),
                    None => i.split(),
                };
                Ok(Self::Garbage(garbage.freeze()))
            }
        }
    }
}

impl Encoder for RawSerialFrame {
    fn write(&self, output: &mut BytesMut) {
        use bake::bytes::{be_u8, slice};

        match self {
            RawSerialFrame::ControlFlow(byte) => be_u8(*byte as u8).write(output),
            RawSerialFrame::Data(data) => slice(data).write(output),
            RawSerialFrame::Garbage(_) => unimplemented!("Garbage is not serializable"),
        }
    }
}

impl SerialFrame {
    pub fn try_into_raw(
        self,
        ctx: &CommandEncodingContext,
    ) -> std::result::Result<RawSerialFrame, EncodingError> {
        match self {
            SerialFrame::ControlFlow(byte) => Ok(RawSerialFrame::ControlFlow(byte)),
            SerialFrame::Command(cmd) => {
                let data = cmd.try_into_raw(ctx)?.as_bytes();
                Ok(RawSerialFrame::Data(data))
            }
            SerialFrame::Raw(data) => Ok(RawSerialFrame::Data(data)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! hex_bytes {
        ($hex:expr) => {
            bytes::BytesMut::from(hex::decode($hex).unwrap().as_slice())
        };
    }

    #[test]
    fn test_garbage() {
        let mut data = hex_bytes!("07080901");
        let expected = hex_bytes!("070809").freeze();
        let remaining = hex_bytes!("01").freeze();
        assert_eq!(
            RawSerialFrame::parse_mut(&mut data),
            Ok(RawSerialFrame::Garbage(expected))
        );
        assert_eq!(data, remaining);
    }

    #[test]
    fn test_data() {
        let mut data = hex_bytes!("01030008f406");
        let expected = hex_bytes!("01030008f4").freeze();
        let remaining = hex_bytes!("06").freeze();
        assert_eq!(
            RawSerialFrame::parse_mut(&mut data),
            Ok(RawSerialFrame::Data(expected))
        );
        assert_eq!(data, remaining);
    }

    #[test]
    fn test_many() {
        let mut data = hex_bytes!("01030008f406180000000801");
        let expected = hex_bytes!("01030008f4").freeze();
        let garbage = hex_bytes!("00000008").freeze();
        let remaining = hex_bytes!("01").freeze();

        let mut results: Vec<RawSerialFrame> = Vec::new();
        while let Ok(frame) = RawSerialFrame::parse_mut(&mut data) {
            results.push(frame);
        }
        assert_eq!(
            results,
            vec![
                RawSerialFrame::Data(expected),
                RawSerialFrame::ControlFlow(ControlFlow::ACK),
                RawSerialFrame::ControlFlow(ControlFlow::CAN),
                RawSerialFrame::Garbage(garbage),
            ]
        );
        assert_eq!(data, remaining);
    }
}
