use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use proc_macros::CCValues;
use zwave_core::encoding::encoders::empty;
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

impl CCSerializable for NoOperationCC {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}
