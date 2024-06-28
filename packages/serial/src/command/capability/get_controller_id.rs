use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use custom_debug_derive::Debug;
use zwave_core::serialize;
use zwave_core::parse::bytes::be_u32;
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetControllerIdRequest {}

impl CommandId for GetControllerIdRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetControllerId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetControllerIdRequest {}

impl CommandRequest for GetControllerIdRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetControllerIdRequest {
    fn parse(_i: &mut Bytes, _ctx: &mut CommandParsingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CommandEncodingContext> for GetControllerIdRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for GetControllerIdRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerIdResponse {
    #[debug(format = "0x{:08x}")]
    pub home_id: u32,
    pub own_node_id: NodeId,
}

impl CommandId for GetControllerIdResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetControllerId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetControllerIdResponse {}

impl CommandParsable for GetControllerIdResponse {
    fn parse(i: &mut Bytes, ctx: &mut CommandParsingContext) -> ParseResult<Self> {
        let home_id = be_u32(i)?;
        let own_node_id = NodeId::parse(i, ctx.node_id_type)?;

        Ok(Self {
            home_id,
            own_node_id,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for GetControllerIdResponse {
    fn serialize(&self, output: &mut BytesMut, ctx: &CommandEncodingContext) {
        use serialize::bytes::be_u32;
        be_u32(self.home_id).serialize(output);
        self.own_node_id.serialize(output, ctx.node_id_type);
    }
}

impl ToLogPayload for GetControllerIdResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("home ID", format!("0x{:08x}", self.home_id))
            .with_entry("own node ID", self.own_node_id.to_string())
            .into()
    }
}
