use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use zwave_core::checksum::crc16_incremental;
use zwave_core::parse::{
    bytes::{be_u16, complete::take},
    combinators::map_res,
    validate,
};
use zwave_core::prelude::*;
use zwave_core::serialize::{self, Serializable};

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
    fn parse(i: &mut Bytes, ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let mut payload = take(i.len() - 2usize).parse(i)?;
        let checksum = be_u16(i)?;

        // The checksum includes the entire CRC16 CC
        let expected_checksum = crc16_incremental()
            .update(&[
                CommandClasses::CRC16Encapsulation as u8,
                Crc16CCCommand::CommandEncapsulation as u8,
            ])
            .update(&payload)
            .get();

        validate(
            checksum == expected_checksum,
            format!(
                "checksum mismatch: expected {:#06x}, got {:#06x}",
                expected_checksum, checksum
            ),
        )?;

        let encapsulated =
            map_res(CCRaw::parse, |raw| CC::try_from_raw(raw, ctx)).parse(&mut payload)?;

        Ok(Self {
            encapsulated: Box::new(encapsulated),
        })
    }
}

impl CCSerializable for Crc16CCCommandEncapsulation {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::{bytes::be_u16, bytes::slice, sequence::tuple};

        let command = self.encapsulated.clone();
        let payload = command.as_raw().as_bytes();

        // The checksum includes the entire CRC16 CC
        let checksum = crc16_incremental()
            .update(&[self.cc_id() as u8, self.cc_command().unwrap()])
            .update(&payload)
            .get();

        tuple((slice(payload), be_u16(checksum))).serialize(output);
    }
}

impl ToLogPayload for Crc16CCCommandEncapsulation {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_nested(self.encapsulated.to_log_payload())
            .into()
    }
}
