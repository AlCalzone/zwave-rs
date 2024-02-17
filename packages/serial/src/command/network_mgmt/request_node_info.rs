use crate::command::ApplicationUpdateRequestPayload;
use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use typed_builder::TypedBuilder;
use zwave_core::bake::{self, Encoder, EncoderWith};
use zwave_core::munch::{bytes::be_u8, combinators::map};
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, TypedBuilder)]
pub struct RequestNodeInfoRequest {
    node_id: NodeId,
}

impl RequestNodeInfoRequest {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }
}

impl CommandId for RequestNodeInfoRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::RequestNodeInfo
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for RequestNodeInfoRequest {}

impl CommandRequest for RequestNodeInfoRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        true
    }

    fn test_callback(&self, callback: &Command) -> bool {
        // The callback for this comes in an ApplicationUpdateRequest
        let Command::ApplicationUpdateRequest(callback) = callback else {
            return false;
        };

        match &callback.payload {
            ApplicationUpdateRequestPayload::NodeInfoReceived { node_id, .. } => {
                node_id == &self.node_id
            }
            ApplicationUpdateRequestPayload::NodeInfoRequestFailed => true,
            _ => false,
        }
    }
}

impl CommandParsable for RequestNodeInfoRequest {
    fn parse(i: &mut Bytes, ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let node_id = NodeId::parse(i, ctx.node_id_type)?;
        Ok(Self { node_id })
    }
}

impl EncoderWith<&CommandEncodingContext> for RequestNodeInfoRequest {
    fn write(&self, output: &mut BytesMut, ctx: &CommandEncodingContext) {
        self.node_id.write(output, ctx.node_id_type);
    }
}

impl ToLogPayload for RequestNodeInfoRequest {
    fn to_log_payload(&self) -> LogPayload {
        // FIXME: Commands that communicate with a node must use the node logger, which puts the node ID in the primary tags
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestNodeInfoResponse {
    was_sent: bool,
}

impl CommandId for RequestNodeInfoResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::RequestNodeInfo
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for RequestNodeInfoResponse {
    fn is_ok(&self) -> bool {
        self.was_sent
    }
}

impl CommandParsable for RequestNodeInfoResponse {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let was_sent = map(be_u8, |x| x > 0).parse(i)?;
        Ok(Self { was_sent })
    }
}

impl EncoderWith<&CommandEncodingContext> for RequestNodeInfoResponse {
    fn write(&self, output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        use bake::bytes::be_u8;
        be_u8(if self.was_sent { 0x01 } else { 0x00 }).write(output);
    }
}

impl ToLogPayload for RequestNodeInfoResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("was sent", self.was_sent)
            .into()
    }
}
