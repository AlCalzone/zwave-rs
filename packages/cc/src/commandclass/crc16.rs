use crate::prelude::*;
use cookie_factory as cf;
use nom::bytes::complete::take;
use nom::combinator::map_res;
use nom::number::complete::be_u16;
use proc_macros::{CCValues, TryFromRepr};
use zwave_core::checksum::crc16_incremental;
use zwave_core::encoding::validate;
use zwave_core::encoding::{self};
use zwave_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum Crc16CCCommand {
    CommandEncapsulation = 0x01,
}

#[derive(Debug, Clone, PartialEq, CCValues)]
pub struct Crc16CCCommandEncapsulation {
    pub encapsulated: Box<CC>,
}

impl Crc16CCCommandEncapsulation {
    pub fn new(encapsulated: CC) -> Self {
        Self {
            encapsulated: Box::new(encapsulated),
        }
    }
}

impl CCBase for Crc16CCCommandEncapsulation {
    fn expects_response(&self) -> bool {
        // The encapsulated CC decides whether a response is expected
        self.encapsulated.expects_response()
    }

    fn test_response(&self, response: &CC) -> bool {
        // The encapsulated CC decides whether the response is the expected one
        let CC::Crc16CCCommandEncapsulation(Crc16CCCommandEncapsulation { encapsulated }) =
            response
        else {
            return false;
        };
        self.encapsulated.test_response(encapsulated)
    }
}

impl CCId for Crc16CCCommandEncapsulation {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::CRC16Encapsulation
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Crc16CCCommand::CommandEncapsulation as _)
    }
}

impl CCParsable for Crc16CCCommandEncapsulation {
    fn parse<'a>(i: encoding::Input<'a>, ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, payload) = take(i.len() - 2usize)(i)?;
        let (i, checksum) = be_u16(i)?;

        // The checksum includes the entire CRC16 CC
        let expected_checksum = crc16_incremental()
            .update(&[
                CommandClasses::CRC16Encapsulation as u8,
                Crc16CCCommand::CommandEncapsulation as u8,
            ])
            .update(payload)
            .get();

        validate(
            i,
            checksum == expected_checksum,
            format!(
                "checksum mismatch: expected {:#06x}, got {:#06x}",
                expected_checksum, checksum
            ),
        )?;

        let (_, encapsulated) = map_res(CCRaw::parse, |raw| CC::try_from_raw(raw, ctx))(payload)?;

        Ok((
            i,
            Self {
                encapsulated: Box::new(encapsulated),
            },
        ))
    }
}

impl CCSerializable for Crc16CCCommandEncapsulation {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u16, combinator::slice, sequence::tuple};
        move |out| {
            let command = self.encapsulated.clone();
            let payload = command
                .try_into_raw()
                .and_then(|raw| raw.try_to_vec())
                .expect("Serializing a CC should not fail");

            // The checksum includes the entire CRC16 CC
            let checksum = crc16_incremental()
                .update(&[self.cc_id() as u8, self.cc_command().unwrap()])
                .update(&payload)
                .get();

            tuple((slice(payload), be_u16(checksum)))(out)
        }
    }
}
