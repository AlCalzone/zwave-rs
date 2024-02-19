use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use zwave_core::prelude::*;

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
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CommandEncodingContext> for SoftResetRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for SoftResetRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}
