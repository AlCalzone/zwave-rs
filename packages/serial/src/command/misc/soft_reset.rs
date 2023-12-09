use crate::prelude::*;
use zwave_core::prelude::*;

use zwave_core::encoding::{self, encoders::empty};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SoftResetRequest {}

impl CommandId for SoftResetRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SoftReset
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for SoftResetRequest {}

impl CommandRequest for SoftResetRequest {
    fn expects_response(&self) -> bool {
        false
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for SoftResetRequest {
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for SoftResetRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}
