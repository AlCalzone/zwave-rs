use crate::prelude::*;
use bytes::Bytes;
use zwave_core::munch::bytes::be_u8;
use zwave_core::munch::combinators::peek;
use zwave_core::munch::complete::{literal, skip, take};
use zwave_core::munch::validate;
use zwave_core::prelude::*;

use crate::{frame::SerialControlByte, util::hex_fmt};
use cookie_factory as cf;
use custom_debug_derive::Debug;

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

impl BytesParsable for CommandRaw {
    fn parse(i: &mut Bytes) -> MunchResult<Self> {
        // Extract the length, while ensuring that the buffer...
        let (_, len, _) = peek((
            // ...starts with SOF
            literal(SerialControlByte::SOF as u8),
            // (read length)
            be_u8(),
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
        let checksum = be_u8().parse(i)?;

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
        Err(MunchError::Incomplete(_)) => panic!("Expected a parser error"),
        Err(_) => (),
    }
}
