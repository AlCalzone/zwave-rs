use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::CCValues;
use zwave_core::prelude::*;

// No Operation CC has no subcommands

#[derive(Debug, Clone, PartialEq, CCValues)]
pub struct NoOperationCC {}

impl CCBase for NoOperationCC {}

impl CCId for NoOperationCC {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::NoOperation
    }

    fn cc_command(&self) -> Option<u8> {
        None
    }
}

impl CCParsable for NoOperationCC {
    fn parse(_i: &mut Bytes, _ctx: &mut CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for NoOperationCC {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for NoOperationCC {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}
