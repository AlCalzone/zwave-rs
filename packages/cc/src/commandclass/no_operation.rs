use crate::prelude::*;
use zwave_core::prelude::*;
use cookie_factory as cf;
use zwave_core::encoding::{self, encoders::empty};

// No Operation CC has no subcommands

#[derive(Debug, Clone, PartialEq)]
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

impl CCRequest for NoOperationCC {
    fn expects_response(&self) -> bool {
        false
    }

    fn test_response(&self, _response: &CC) -> bool {
        false
    }
}

impl CCParsable for NoOperationCC {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CCSerializable for NoOperationCC {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}
