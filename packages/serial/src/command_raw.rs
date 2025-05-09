use crate::frame::SerialFrame;
use crate::util::with_hex_fmt;
use crate::frame::SerialControlByte;
use bytes::{Bytes, BytesMut};
use std::fmt::Debug;
use zwave_core::prelude::*;
use zwave_core::serialize;
use zwave_core::{
    checksum::xor_sum,
    parse::{
        bytes::{
            be_u8,
            complete::{literal, skip, take},
        },
        combinators::peek,
        validate,
    },
};

#[derive(Clone, PartialEq)]
pub struct CommandRaw {
    pub command_type: CommandType,
    pub function_type: FunctionType,
    pub payload: Bytes,
    pub checksum: u8,
}

impl Debug for CommandRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRaw")
            .field("command_type", &self.command_type)
            .field("function_type", &self.function_type)
            .field("payload", &with_hex_fmt(&self.payload))
            .field("checksum", &format_args!("{:#04x}", &self.checksum))
            .finish()
    }
}

fn command_checksum(cmd_buffer: &[u8]) -> u8 {
    xor_sum(&cmd_buffer[1..cmd_buffer.len() - 1])
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

        let expected_checksum = command_checksum(&raw_data);
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
        let checksum = command_checksum(&buf);
        // Then update the checksum in the buffer
        let len = buf.len();
        buf[len - 1] = checksum;

        slice(buf).serialize(output);
    }
}

impl From<CommandRaw> for SerialFrame {
    fn from(val: CommandRaw) -> Self {
        SerialFrame::Command(val)
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
