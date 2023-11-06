pub mod definitions;
use custom_debug_derive::Debug;

use nom::{
    bytes::streaming::{tag, take},
    combinator::peek,
    number::streaming::be_u8,
    sequence::tuple,
};

use crate::{
    command::definitions::{CommandType, FunctionType},
    frame::{SerialControlByte, Serialize},
    parse::{self, fail_validation},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub command_type: CommandType,
    pub function_type: FunctionType,
    #[debug(with = "hex_fmt")]
    pub payload: Vec<u8>,
    #[debug(format = "{:#04x}")]
    pub checksum: u8,
}

fn hex_fmt<T: std::fmt::Debug + AsRef<[u8]>>(
    n: &T,
    f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    write!(f, "0x{}", hex::encode(n))
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

impl Command {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
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
        if checksum != expected_checksum {
            return fail_validation(
                rem,
                format!(
                    "checksum mismatch: expected {:#04x}, got {:#04x}",
                    expected_checksum, checksum
                ),
            );
        }

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

impl Serialize for Command {
    fn serialize(&self) -> Vec<u8> {
        let mut result = vec![
            SerialControlByte::SOF as u8,
            self.payload.len() as u8 + 3,
            self.command_type as u8,
            self.function_type as u8,
        ];
        result.append(&mut self.payload.clone());
        result.push(self.checksum);
        result
    }
}
