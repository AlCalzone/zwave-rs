use crate::{command_raw::CommandRaw, parse::{self, Serializable}};

use cookie_factory as cf;
use derive_try_from_primitive::*;
use nom::{
    branch::alt,
    bytes::streaming::{tag, take, take_till1},
    combinator::{map, peek, value},
    error::context,
    number::streaming::be_u8,
    sequence::tuple,
};

pub const ACK_BUFFER: [u8; 1] = [SerialControlByte::ACK as u8];
pub const NAK_BUFFER: [u8; 1] = [SerialControlByte::NAK as u8];
pub const CAN_BUFFER: [u8; 1] = [SerialControlByte::CAN as u8];

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum SerialControlByte {
    SOF = 0x01,
    ACK = 0x06,
    NAK = 0x15,
    CAN = 0x18,
}

/// A raw serial frame, as received from the serial port
#[derive(Clone, Debug, PartialEq)]
pub enum RawSerialFrame {
    ACK,
    NAK,
    CAN,
    Data(Vec<u8>),
    Garbage(Vec<u8>),
}

/// A parsed serial frame that contains a control-flow byte or a Serial API command
#[derive(Clone, Debug, PartialEq)]
pub enum SerialFrame {
    ACK,
    NAK,
    CAN,
    Command(CommandRaw),
    Raw(Vec<u8>),
}

fn consume_garbage(i: parse::Input) -> parse::Result<RawSerialFrame> {
    map(
        take_till1(|b| SerialControlByte::try_from(b).is_ok()),
        |g: &[u8]| RawSerialFrame::Garbage(g.to_vec()),
    )(i)
}

fn parse_control(i: parse::Input) -> parse::Result<RawSerialFrame> {
    alt((
        value(RawSerialFrame::ACK, tag(&ACK_BUFFER)),
        value(RawSerialFrame::NAK, tag(&NAK_BUFFER)),
        value(RawSerialFrame::CAN, tag(&CAN_BUFFER)),
    ))(i)
}

fn parse_data(i: parse::Input) -> parse::Result<RawSerialFrame> {
    // Ensure that the buffer contains at least 5 bytes
    peek(take(5usize))(i)?;

    // Ensure that it starts with a SOF byte and extract the length of the rest of the command
    let (_, (_, len)) = peek(tuple((tag([SerialControlByte::SOF as u8]), be_u8)))(i)?;

    // Take the whole command
    let (i, data) = take(len + 2)(i)?;

    // And return the whole thing
    Ok((i, RawSerialFrame::Data(data.to_vec())))
}

impl RawSerialFrame {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        // A serial frame is either a control byte, data starting with SOF, or skipped garbage
        context(
            "Serial Frame",
            alt((consume_garbage, parse_control, parse_data)),
        )(i)
    }

    pub fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice};

        move |out| match self {
            RawSerialFrame::ACK => be_u8(SerialControlByte::ACK as u8)(out),
            RawSerialFrame::NAK => be_u8(SerialControlByte::NAK as u8)(out),
            RawSerialFrame::CAN => be_u8(SerialControlByte::CAN as u8)(out),
            RawSerialFrame::Data(data) => slice(data)(out),
            RawSerialFrame::Garbage(_) => unimplemented!("Garbage is not serializable"),
        }
    }
}

impl_vec_conversion_for!(RawSerialFrame);

impl SerialFrame {
    pub fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice};

        move |out| match self {
            SerialFrame::ACK => be_u8(SerialControlByte::ACK as u8)(out),
            SerialFrame::NAK => be_u8(SerialControlByte::NAK as u8)(out),
            SerialFrame::CAN => be_u8(SerialControlByte::CAN as u8)(out),
            SerialFrame::Command(cmd) => cmd.serialize()(out),
            SerialFrame::Raw(data) => slice(data)(out),
        }
    }
}

impl Into<RawSerialFrame> for SerialFrame {
    fn into(self) -> RawSerialFrame {
        match self {
            SerialFrame::ACK => RawSerialFrame::ACK,
            SerialFrame::NAK => RawSerialFrame::NAK,
            SerialFrame::CAN => RawSerialFrame::CAN,
            SerialFrame::Command(cmd) => RawSerialFrame::Data((&cmd).try_into().unwrap()),
            SerialFrame::Raw(data) => RawSerialFrame::Data(data),
        }
    }
}

impl_vec_serializing_for!(SerialFrame);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_garbage() {
        let data = hex::decode("07080901").unwrap();
        let expected = hex::decode("070809").unwrap();
        let remaining = hex::decode("01").unwrap();
        assert_eq!(
            consume_garbage(&data),
            Ok((remaining.as_slice(), RawSerialFrame::Garbage(expected)))
        );
    }

    #[test]
    fn test_control() {
        let data = hex::decode("0606151801").unwrap();
        let remaining = hex::decode("01").unwrap();
        assert_eq!(
            nom::multi::many0(parse_control)(&data),
            Ok((
                remaining.as_slice(),
                vec![
                    RawSerialFrame::ACK,
                    RawSerialFrame::ACK,
                    RawSerialFrame::NAK,
                    RawSerialFrame::CAN,
                ]
            )),
        );
    }

    #[test]
    fn test_data() {
        let data = hex::decode("01030008f406").unwrap();
        let expected = hex::decode("01030008f4").unwrap();
        let remaining = hex::decode("06").unwrap();
        assert_eq!(
            parse_data(&data),
            Ok((remaining.as_slice(), RawSerialFrame::Data(expected),))
        );
    }

    #[test]
    fn test_many() {
        let data = hex::decode("01030008f406180000000801").unwrap();
        let expected = hex::decode("01030008f4").unwrap();
        let garbage = hex::decode("00000008").unwrap();

        let mut results: Vec<RawSerialFrame> = Vec::new();
        let mut input = data.as_slice();
        while let Ok((remaining, frame)) = RawSerialFrame::parse(input) {
            results.push(frame);
            input = remaining;
        }
        assert_eq!(input, vec![0x01]);
        assert_eq!(
            results,
            vec![
                RawSerialFrame::Data(expected),
                RawSerialFrame::ACK,
                RawSerialFrame::CAN,
                RawSerialFrame::Garbage(garbage),
            ]
        );
    }
}
