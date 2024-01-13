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
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CommandEncodingContext) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CommandSerializable for SoftResetRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self, _ctx: &'a CommandEncodingContext) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl ToLogPayload for SoftResetRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}
