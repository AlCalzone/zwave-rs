use crate::prelude::*;
use zwave_core::prelude::*;

use zwave_core::encoding::{self, validate};

use crate::{frame::SerialControlByte, util::hex_fmt};
use cookie_factory as cf;
use custom_debug_derive::Debug;
use nom::{
    bytes::complete::{tag, take},
    combinator::peek,
    number::complete::be_u8,
    sequence::tuple,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandRaw {
    pub command_type: CommandType,
    pub function_type: FunctionType,
    #[debug(with = "hex_fmt")]
    pub payload: Vec<u8>,
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

impl CommandRaw {
    fn serialize_no_checksum<'a, W: std::io::Write + 'a>(
        &'a self,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice, sequence::tuple};

        let sof = be_u8(SerialControlByte::SOF as u8);
        let len = be_u8(self.payload.len() as u8 + 3);
        let command_type = self.command_type.serialize();
        let function_type = self.function_type.serialize();
        let payload = slice(&self.payload);
        let checksum = be_u8(0); // placeholder

        tuple((sof, len, command_type, function_type, payload, checksum))
    }
}

impl Parsable for CommandRaw {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        // Ensure that the buffer contains at least 5 bytes
        peek(take(5usize))(i)?;

        // Ensure that it starts with a SOF byte and extract the length of the rest of the command
        let (_, (_, len)) = peek(tuple((tag([SerialControlByte::SOF as u8]), be_u8)))(i)?;
        let (rem, raw_data) = peek(take(len + 2))(i)?;

        // Skip the SOF and length bytes
        let (i, _) = take(2usize)(i)?;

        let (i, command_type) = CommandType::parse(i)?;
        let (i, function_type) = FunctionType::parse(i)?;
        let (i, payload) = take(len - 3)(i)?;
        let (i, checksum) = be_u8(i)?;

        let expected_checksum = compute_checksum(raw_data);
        validate(
            rem,
            checksum == expected_checksum,
            format!(
                "checksum mismatch: expected {:#04x}, got {:#04x}",
                expected_checksum, checksum
            ),
        )?;

        Ok((
            i,
            Self {
                command_type,
                function_type,
                payload: payload.to_vec(),
                checksum,
            },
        ))
    }
}

impl Serializable for CommandRaw {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice};

        // First serialize the command without checksum,
        move |out| {
            let mut buf = cf::gen_simple(self.serialize_no_checksum(), Vec::new())?;
            let checksum = compute_checksum(&buf);
            // then write the checksum into the last byte
            let len = buf.len();
            cf::gen_simple(be_u8(checksum), &mut buf[len - 1..])?;
            slice(buf)(out)
        }
    }
}

#[test]
fn test_parse_invalid_checksum() {
    // This is an actual message with a correct checksum
    let input = hex::decode("01030002fe").unwrap();
    let result = CommandRaw::try_from(input.as_ref());
    assert!(result.is_ok());

    // Now it is wrong
    let input = hex::decode("01030002ff").unwrap();
    let result = CommandRaw::try_from(input.as_ref());
    match result {
        Ok(_) => panic!("Expected an error"),
        Err(EncodingError::Parse(_)) => (),
        Err(_) => panic!("Expected a parser error"),
    }
}
