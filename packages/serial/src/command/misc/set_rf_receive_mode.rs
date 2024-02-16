use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use typed_builder::TypedBuilder;
use zwave_core::munch::{bytes::be_u8, combinators::map};
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, TypedBuilder)]
pub struct SetRfReceiveModeRequest {
    // Whether the Z-Wave module's RF receiver should be enabled
    enabled: bool,
}

impl CommandId for SetRfReceiveModeRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetRFReceiveMode
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for SetRfReceiveModeRequest {}

impl CommandRequest for SetRfReceiveModeRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for SetRfReceiveModeRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        eprintln!("ERROR: SetRfReceiveModeRequest::parse() not implemented");
        Ok(Self::default())
    }
}

impl CommandSerializable for SetRfReceiveModeRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(if self.enabled { 1 } else { 0 })
    }
}

impl ToLogPayload for SetRfReceiveModeRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("enabled", self.enabled)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetRfReceiveModeResponse {
    success: bool,
}

impl CommandId for SetRfReceiveModeResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetRFReceiveMode
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for SetRfReceiveModeResponse {
    fn is_ok(&self) -> bool {
        self.success
    }
}

impl CommandParsable for SetRfReceiveModeResponse {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let success = map(be_u8, |x| x > 0).parse(i)?;
        Ok(Self { success })
    }
}

impl CommandSerializable for SetRfReceiveModeResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: SetRfReceiveModeResponse::serialize() not implemented")
    }
}

impl ToLogPayload for SetRfReceiveModeResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("success", self.success)
            .into()
    }
}
