use crate::parse;
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

#[derive(Clone, Debug, PartialEq)]
pub enum SerialFrame {
    ACK,
    NAK,
    CAN,
    Data(Vec<u8>),
    Garbage(Vec<u8>),
}

fn consume_garbage(i: parse::Input) -> parse::Result<SerialFrame> {
    map(
        take_till1(|b| SerialControlByte::try_from(b).is_ok()),
        |g: &[u8]| SerialFrame::Garbage(g.to_vec()),
    )(i)
}

fn parse_control(i: parse::Input) -> parse::Result<SerialFrame> {
    alt((
        value(SerialFrame::ACK, tag(&ACK_BUFFER)),
        value(SerialFrame::NAK, tag(&NAK_BUFFER)),
        value(SerialFrame::CAN, tag(&CAN_BUFFER)),
    ))(i)
}

fn parse_data(i: parse::Input) -> parse::Result<SerialFrame> {
    // Ensure that the buffer contains at least 5 bytes
    peek(take(5usize))(i)?;

    // Ensure that it starts with a SOF byte and extract the length of the rest of the command
    let (_, (_, len)) = peek(tuple((tag([SerialControlByte::SOF as u8]), be_u8)))(i)?;

    // Take the whole command
    let (i, data) = take(len + 2)(i)?;

    // And return the whole thing
    Ok((i, SerialFrame::Data(data.to_vec())))
}

impl SerialFrame {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        // A serial frame is either a control byte, data starting with SOF, or skipped garbage
        context(
            "Serial Frame",
            alt((consume_garbage, parse_control, parse_data)),
        )(i)
    }
}

impl Into<Vec<u8>> for &SerialFrame {
    fn into(self) -> Vec<u8> {
        match self {
            SerialFrame::ACK => Vec::from(ACK_BUFFER),
            SerialFrame::NAK => Vec::from(NAK_BUFFER),
            SerialFrame::CAN => Vec::from(CAN_BUFFER),
            SerialFrame::Data(data) => data.clone(),
            SerialFrame::Garbage(data) => data.clone(),
        }
    }
}

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

// Adds convenience for all references that can be converted into a Vec<u8>
// This way, one can call `a.serialize()`, rather than `(&a).into()`
impl<T> Serialize for T
where
    for<'a> &'a T: Into<Vec<u8>>,
{
    fn serialize(&self) -> Vec<u8> {
        self.into()
    }
}

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
            Ok((remaining.as_slice(), SerialFrame::Garbage(expected)))
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
                    SerialFrame::ACK,
                    SerialFrame::ACK,
                    SerialFrame::NAK,
                    SerialFrame::CAN,
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
            Ok((remaining.as_slice(), SerialFrame::Data(expected),))
        );
    }

    #[test]
    fn test_many() {
        let data = hex::decode("01030008f406180000000801").unwrap();
        let expected = hex::decode("01030008f4").unwrap();
        let garbage = hex::decode("00000008").unwrap();

        let mut results: Vec<SerialFrame> = Vec::new();
        let mut input = data.as_slice();
        while let Ok((remaining, frame)) = SerialFrame::parse(input) {
            results.push(frame);
            input = remaining;
        }
        assert_eq!(input, vec![0x01]);
        assert_eq!(
            results,
            vec![
                SerialFrame::Data(expected),
                SerialFrame::ACK,
                SerialFrame::CAN,
                SerialFrame::Garbage(garbage),
            ]
        );
    }
}
