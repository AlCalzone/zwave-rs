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
    fn parse(_i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::munch::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl CCEncoder for NoOperationCC {
    fn write(&self, _output: &mut BytesMut) {
        // No payload
    }
}
