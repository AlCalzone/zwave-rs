use std::fmt::Display;

use crate::prelude::{Command, CommandEncodingContext};
use bytes::BytesMut;
use cookie_factory as cf;
use proc_macros::TryFromRepr;
use zwave_core::encoding::{self, BytesParsable};
use zwave_core::munch::{self, combinators::*, streaming::*, Parser};
use zwave_core::prelude::*;

#[derive(Debug, TryFromRepr)]
#[repr(u8)]
pub enum SerialControlByte {
    SOF = 0x01,
    ACK = 0x06,
    NAK = 0x15,
    CAN = 0x18,
}

pub const ACK_BUFFER: [u8; 1] = [SerialControlByte::ACK as u8];
pub const NAK_BUFFER: [u8; 1] = [SerialControlByte::NAK as u8];
pub const CAN_BUFFER: [u8; 1] = [SerialControlByte::CAN as u8];

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
    Data(Vec<u8>),
    Garbage(Vec<u8>),
}

/// A parsed serial frame that contains a control-flow byte or a Serial API command
#[derive(Clone, Debug, PartialEq)]
pub enum SerialFrame {
    ControlFlow(ControlFlow),
    Command(Command),
    Raw(Vec<u8>),
}

fn consume_garbage() -> impl munch::Parser<RawSerialFrame> {
    map(
        take_while1(|b| SerialControlByte::try_from(b).is_err()),
        |g: BytesMut| RawSerialFrame::Garbage(g.to_vec()),
    )
}

fn parse_control() -> impl munch::Parser<RawSerialFrame> {
    move |i: &mut BytesMut| {
        if let Ok(_) = literal(SerialControlByte::ACK as u8).parse_peek(i) {
            return Ok(RawSerialFrame::ControlFlow(ControlFlow::ACK));
        }

        if let Ok(_) = literal(SerialControlByte::NAK as u8).parse_peek(i) {
            return Ok(RawSerialFrame::ControlFlow(ControlFlow::NAK));
        }

        // Always consume the first byte, even in case of failure
        if let Ok(_) = literal(SerialControlByte::CAN as u8).parse(i) {
            return Ok(RawSerialFrame::ControlFlow(ControlFlow::CAN));
        }

        Err(munch::ParseError::Recoverable(()))
    }
    // FIXME: Implement alt() combinator and use it
    // alt((
    //     value(
    //         RawSerialFrame::ControlFlow(ControlFlow::ACK),
    //         tag(&ACK_BUFFER),
    //     ),
    //     value(
    //         RawSerialFrame::ControlFlow(ControlFlow::NAK),
    //         tag(&NAK_BUFFER),
    //     ),
    //     value(
    //         RawSerialFrame::ControlFlow(ControlFlow::CAN),
    //         tag(&CAN_BUFFER),
    //     ),
    // ))(i)
}

fn parse_data() -> impl munch::Parser<RawSerialFrame> {
    move |i: &mut BytesMut| {
        let checkpoint = i.clone();

        // Ensure that the buffer contains at least 5 bytes
        peek(take(5usize)).parse(i)?;

        // FIXME: Implement tuple() combinator and use it

        literal(SerialControlByte::SOF as u8).parse(i)?;
        let len = map(take(1usize), |b| b[0]).parse(i)?;

        *i = checkpoint.clone();

        // FIXME: Implement u8() parser and use it
        let data = take(len + 2).parse(i)?;

        Ok(RawSerialFrame::Data(data.to_vec()))
    }

    // // Ensure that it starts with a SOF byte and extract the length of the rest of the command
    // let (_, (_, len)) = peek(tuple((tag([SerialControlByte::SOF as u8]), be_u8)))(i)?;

    // // Take the whole command
    // let (i, data) = take(len + 2)(i)?;

    // // And return the whole thing
    // Ok((i, RawSerialFrame::Data(data.to_vec())))
}

impl BytesParsable for RawSerialFrame {
    fn parse(i: &mut BytesMut) -> munch::ParseResult<Self> {
        // A serial frame is either a control byte, data starting with SOF, or skipped garbage

        if let Ok(garbage) = consume_garbage().parse_peek(i) {
            return Ok(garbage);
        }

        if let Ok(control) = parse_control().parse_peek(i) {
            return Ok(control);
        }

        parse_data().parse(i)

        // FIXME: Implement alt() combinator and use it
        // context(
        //     "Serial Frame",
        //     alt((consume_garbage, parse_control, parse_data)),
        // )(i)
    }
}

impl Serializable for RawSerialFrame {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice};

        move |out| match self {
            RawSerialFrame::ControlFlow(byte) => be_u8(*byte as u8)(out),
            RawSerialFrame::Data(data) => slice(data)(out),
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
            SerialFrame::Command(cmd) => cmd
                .try_into_raw(ctx)?
                .try_to_vec()
                .map(RawSerialFrame::Data),
            SerialFrame::Raw(data) => Ok(RawSerialFrame::Data(data)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! hex_bytes {
        ($hex:expr) => {
            BytesMut::from(hex::decode($hex).unwrap().as_slice())
        };
    }

    macro_rules! hex_vec {
        ($hex:expr) => {
            hex::decode($hex).unwrap()
        };
    }

    #[test]
    fn test_garbage() {
        let mut data = hex_bytes!("07080901");
        let expected = hex_vec!("070809");
        let remaining = hex_bytes!("01");
        assert_eq!(
            consume_garbage().parse(&mut data),
            Ok(RawSerialFrame::Garbage(expected))
        );
        assert_eq!(data, remaining);
    }

    #[test]
    fn test_control() {
        let mut data = hex_bytes!("0606151801");
        let remaining = hex_bytes!("01");
        assert_eq!(
            munch::multi::many_0(parse_control()).parse(&mut data),
            Ok(vec![
                RawSerialFrame::ControlFlow(ControlFlow::ACK),
                RawSerialFrame::ControlFlow(ControlFlow::ACK),
                RawSerialFrame::ControlFlow(ControlFlow::NAK),
                RawSerialFrame::ControlFlow(ControlFlow::CAN),
            ]),
        );
        assert_eq!(data, remaining);
    }

    #[test]
    fn test_data() {
        let mut data = hex_bytes!("01030008f406");
        let expected = hex_vec!("01030008f4");
        let remaining = hex_bytes!("06");
        assert_eq!(
            parse_data().parse(&mut data),
            Ok(RawSerialFrame::Data(expected))
        );
        assert_eq!(data, remaining);
    }

    #[test]
    fn test_many() {
        let mut data = hex_bytes!("01030008f406180000000801");
        let expected = hex_vec!("01030008f4");
        let garbage = hex_vec!("00000008");
        let remaining = hex_bytes!("01");

        let mut results: Vec<RawSerialFrame> = Vec::new();
        while let Ok(frame) = RawSerialFrame::parse(&mut data) {
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
