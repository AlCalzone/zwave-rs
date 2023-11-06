use nom::{
    branch::alt,
    bytes::streaming::{tag, take, take_till1},
    combinator::{map, peek, value},
    error::context,
    number::streaming::be_u8,
    sequence::tuple,
};

use crate::{consts::SerialAPIControlByte, error::Result, parse};

#[derive(Clone, Debug, PartialEq)]
pub enum SerialAPIFrame {
    ACK,
    NAK,
    CAN,
    Command(SerialAPICommand),
    Garbage(Vec<u8>),
}

pub const ACK_BUFFER: [u8; 1] = [SerialAPIControlByte::ACK as u8];
pub const NAK_BUFFER: [u8; 1] = [SerialAPIControlByte::NAK as u8];
pub const CAN_BUFFER: [u8; 1] = [SerialAPIControlByte::CAN as u8];

#[derive(Clone, Debug, PartialEq)]
pub struct SerialAPICommand {
    data: Vec<u8>,
}

impl AsRef<[u8]> for SerialAPIFrame {
    fn as_ref(&self) -> &[u8] {
        match &self {
            SerialAPIFrame::ACK => &ACK_BUFFER,
            SerialAPIFrame::NAK => &NAK_BUFFER,
            SerialAPIFrame::CAN => &CAN_BUFFER,
            SerialAPIFrame::Command(cmd) => cmd.as_ref(),
            SerialAPIFrame::Garbage(data) => &data,
        }
    }
}

impl SerialAPICommand {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn payload(&self) -> &[u8] {
        &self.data[2..&self.data.len() - 1]
    }

    pub fn checksum(&self) -> u8 {
        *self.data.last().unwrap()
    }
}

impl AsRef<[u8]> for SerialAPICommand {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

fn consume_garbage(i: parse::Input) -> parse::Result<SerialAPIFrame> {
    map(
        take_till1(|b| SerialAPIControlByte::try_from(b).is_ok()),
        |g: &[u8]| SerialAPIFrame::Garbage(g.to_vec()),
    )(i)
}

fn parse_control(i: parse::Input) -> parse::Result<SerialAPIFrame> {
    alt((
        value(SerialAPIFrame::ACK, tag(&ACK_BUFFER)),
        value(SerialAPIFrame::NAK, tag(&NAK_BUFFER)),
        value(SerialAPIFrame::CAN, tag(&CAN_BUFFER)),
    ))(i)
}

fn parse_command(i: parse::Input) -> parse::Result<SerialAPIFrame> {
    // Ensure that the buffer contains at least 5 bytes
    peek(take(5usize))(i)?;

    // Ensure that it starts with a SOF byte and extract the length of the rest of the command
    let (_, (_, len)) = peek(tuple((tag([SerialAPIControlByte::SOF as u8]), be_u8)))(i)?;

    // Take the whole command
    let (i, data) = take(len + 2)(i)?;

    // And return the whole thing
    Ok((
        i,
        SerialAPIFrame::Command(SerialAPICommand::new(data.to_vec())),
    ))
}

#[test]
fn test_garbage() {
    let data = hex::decode("07080901").unwrap();
    let expected = hex::decode("070809").unwrap();
    let remaining = hex::decode("01").unwrap();
    assert_eq!(
        consume_garbage(&data),
        Ok((remaining.as_slice(), SerialAPIFrame::Garbage(expected)))
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
                SerialAPIFrame::ACK,
                SerialAPIFrame::ACK,
                SerialAPIFrame::NAK,
                SerialAPIFrame::CAN,
            ]
        )),
    );
}

#[test]
fn test_command() {
    let data = hex::decode("01030008f406").unwrap();
    let expected = hex::decode("01030008f4").unwrap();
    let remaining = hex::decode("06").unwrap();
    assert_eq!(
        parse_command(&data),
        Ok((
            remaining.as_slice(),
            SerialAPIFrame::Command(SerialAPICommand { data: expected }),
        ))
    );
}

#[test]
fn test_many() {
    let data = hex::decode("01030008f406180000000801").unwrap();
    let expected = hex::decode("01030008f4").unwrap();
    let garbage = hex::decode("00000008").unwrap();

    let mut results: Vec<SerialAPIFrame> = Vec::new();
    let mut input = data.as_slice();
    while let Ok((remaining, frame)) = SerialAPIFrame::parse(input) {
        results.push(frame);
        input = remaining;
    }
    assert_eq!(input, vec![0x01]);
    assert_eq!(
        results,
        vec![
            SerialAPIFrame::Command(SerialAPICommand { data: expected }),
            SerialAPIFrame::ACK,
            SerialAPIFrame::CAN,
            SerialAPIFrame::Garbage(garbage),
        ]
    );
}

impl SerialAPIFrame {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        // A serial API frame is either a control byte or a command, or skipped garbage data
        context(
            "Serial API Frame",
            alt((consume_garbage, parse_control, parse_command)),
        )(i)
    }
}

pub type SerialAPIListener = crossbeam_channel::Receiver<SerialAPIFrame>;
pub trait SerialAPIWriter<'a> {
    // FIXME: Do not accept garbage here
    fn write(&self, frame: SerialAPIFrame) -> Result<()>;
    fn write_raw(&self, data: impl AsRef<[u8]>) -> Result<()>;
}

pub trait PortBinding {
    type Open;

    fn new(path: &str) -> Self;

    fn open(self) -> Result<Self::Open>;
}

pub trait OpenPortBinding {
    type Closed;

    fn close(self) -> Result<Self::Closed>;
    fn listener(&self) -> SerialAPIListener;
    fn writer<'a>(&self) -> impl SerialAPIWriter + Clone;

    // fn write(&mut self, data: Vec<u8>) -> Result<()>;
}
