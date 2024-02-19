use crate::{frame::SerialControlByte, util::hex_fmt};
use bytes::{Bytes, BytesMut};
use custom_debug_derive::Debug;
use zwave_core::parse::{
    bytes::{
        be_u8,
        complete::{literal, skip, take},
    },
    combinators::peek,
    validate,
};
use zwave_core::prelude::*;
use zwave_core::serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct CommandRaw {
    pub command_type: CommandType,
    pub function_type: FunctionType,
    #[debug(with = "hex_fmt")]
    pub payload: Bytes,
    #[debug(format = "{:#04x}")]
    pub checksum: u8,
}

fn compute_checksum(data: &[u8]) -> u8 {
    data[1..data.len() - 1].iter().fold(0xff, |acc, x| acc ^ x)
}

#[test]
fn test_checksum() {
    // This is an actual message with a correct checksum
    let input = hex::decode("01030002fe").unwrap();
    let expected = 0xfe;
    assert_eq!(compute_checksum(&input), expected);
}

impl Parsable for CommandRaw {
    fn parse(i: &mut Bytes) -> ParseResult<Self> {
        // Extract the length, while ensuring that the buffer...
        let (_, len, _) = peek((
            // ...starts with SOF
            literal(SerialControlByte::SOF as u8),
            // (read length)
            be_u8,
            // ...and contains at least 5 bytes
            take(3usize),
        ))
        .parse(i)?;

        // Remember a copy of the command buffer for the checksum later
        let raw_data: Bytes = i.clone().split_to(len as usize + 2);

        // Skip the SOF and length bytes
        skip(2usize).parse(i)?;

        let command_type = CommandType::parse(i)?;
        let function_type = FunctionType::parse(i)?;
        let payload = take(len - 3).parse(i)?;
        let checksum = be_u8(i)?;

        let expected_checksum = compute_checksum(&raw_data);
        validate(
            checksum == expected_checksum,
            format!(
                "checksum mismatch: expected {:#04x}, got {:#04x}",
                expected_checksum, checksum
            ),
        )?;

        Ok(Self {
            command_type,
            function_type,
            payload,
            checksum,
        })
    }
}

impl CommandRaw {
    fn serialize_no_checksum(&self) -> impl Serializable + '_ {
        use serialize::{
            bytes::{be_u8, slice},
            sequence::tuple,
        };

        let sof = be_u8(SerialControlByte::SOF as u8);
        let len = be_u8(self.payload.len() as u8 + 3);
        let payload = slice(&self.payload);
        let checksum = be_u8(0); // placeholder

        tuple((
            sof,
            len,
            self.command_type,
            self.function_type,
            payload,
            checksum,
        ))
    }
}

impl Serializable for CommandRaw {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::slice;

        let mut buf = self.serialize_no_checksum().as_bytes_mut();
        let checksum = compute_checksum(&buf);
        // Then update the checksum in the buffer
        let len = buf.len();
        buf[len - 1] = checksum;

        slice(buf).serialize(output);
    }
}

#[test]
fn test_parse_invalid_checksum() {
    macro_rules! hex_bytes {
        ($hex:expr) => {
            bytes::BytesMut::from(hex::decode($hex).unwrap().as_slice()).freeze()
        };
    }

    // This is an actual message with a correct checksum
    let mut input = hex_bytes!("01030002fe");
    let result = CommandRaw::parse(&mut input);
    assert!(result.is_ok());

    // Now it is wrong
    let mut input = hex_bytes!("01030002ff");
    let result = CommandRaw::parse(&mut input);
    match result {
        Ok(_) => panic!("Expected an error"),
        Err(ParseError::Incomplete(_)) => panic!("Expected a parser error"),
        Err(_) => (),
    }
}

#[test]
fn test_serialize() {
    let cmd = CommandRaw {
        command_type: CommandType::Request,
        function_type: FunctionType::GetSerialApiInitData,
        payload: Bytes::new(),
        checksum: 0u8,
    };

    macro_rules! hex_bytes {
        ($hex:expr) => {
            bytes::BytesMut::from(hex::decode($hex).unwrap().as_slice()).freeze()
        };
    }

    let expected = hex_bytes!("01030002fe");
    let actual = cmd.as_bytes_mut();
    assert_eq!(actual, expected);
}
