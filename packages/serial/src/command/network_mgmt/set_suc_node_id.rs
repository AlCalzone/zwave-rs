use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use typed_builder::TypedBuilder;
use zwave_core::parse::{bytes::be_u8, combinators::map, parser_not_implemented};
use zwave_core::prelude::*;
use zwave_core::serialize::{self, Serializable, SerializableWith};

#[derive(Default, Debug, Clone, PartialEq, TypedBuilder)]
pub struct SetSucNodeIdRequest {
    // Needed for knowing whether a callback is expected
    own_node_id: NodeId,
    suc_node_id: NodeId,
    enable_suc: bool,
    enable_sis: bool,
    #[builder(setter(skip), default)]
    callback_id: Option<u8>,
    #[builder(default)]
    transmit_options: TransmitOptions,
}

impl CommandId for SetSucNodeIdRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetSUCNodeId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for SetSucNodeIdRequest {
    fn callback_id(&self) -> Option<u8> {
        self.callback_id
    }
}

impl CommandRequest for SetSucNodeIdRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        self.suc_node_id == self.own_node_id
    }

    fn needs_callback_id(&self) -> bool {
        true
    }

    fn set_callback_id(&mut self, callback_id: Option<u8>) {
        self.callback_id = callback_id;
    }
}

impl CommandParsable for SetSucNodeIdRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> ParseResult<Self> {
        parser_not_implemented("ERROR: SetSucNodeIdRequest::parse() not implemented")
        // Ok(Self {})
    }
}

impl SerializableWith<&CommandEncodingContext> for SetSucNodeIdRequest {
    fn serialize(&self, output: &mut BytesMut, ctx: &CommandEncodingContext) {
        use serialize::{bytes::be_u8, sequence::tuple};

        self.suc_node_id.serialize(output, ctx.node_id_type);
        tuple((
            be_u8(if self.enable_suc { 0x01 } else { 0x00 }),
            self.transmit_options,
            be_u8(if self.enable_sis { 0x01 } else { 0x00 }),
            be_u8(self.callback_id.unwrap_or(0)),
        ))
        .serialize(output)
    }
}

impl ToLogPayload for SetSucNodeIdRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("SUC node ID", self.suc_node_id.to_string())
            .with_entry("enable SUC", self.enable_suc)
            .with_entry("enable SIS", self.enable_sis)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetSucNodeIdResponse {
    was_executed: bool,
}

impl CommandId for SetSucNodeIdResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetSUCNodeId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for SetSucNodeIdResponse {
    fn is_ok(&self) -> bool {
        self.was_executed
    }
}

impl CommandParsable for SetSucNodeIdResponse {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> ParseResult<Self> {
        let was_executed = map(be_u8, |x| x > 0).parse(i)?;
        Ok(Self { was_executed })
    }
}

impl SerializableWith<&CommandEncodingContext> for SetSucNodeIdResponse {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        use serialize::bytes::be_u8;
        be_u8(if self.was_executed { 0x01 } else { 0x00 }).serialize(output)
    }
}

impl ToLogPayload for SetSucNodeIdResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("was executed", self.was_executed)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetSucNodeIdCallback {
    callback_id: Option<u8>,
    success: bool,
}

impl CommandId for SetSucNodeIdCallback {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetSUCNodeId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for SetSucNodeIdCallback {
    fn callback_id(&self) -> Option<u8> {
        self.callback_id
    }

    fn is_ok(&self) -> bool {
        self.success
    }
}

impl CommandParsable for SetSucNodeIdCallback {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> ParseResult<Self> {
        let callback_id = be_u8(i)?;
        let status = be_u8(i)?;

        // Status is either 0x05 (success) or 0x06 (failure)

        Ok(Self {
            callback_id: Some(callback_id),
            success: status == 0x05,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for SetSucNodeIdCallback {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: SetSucNodeIdCallback::write() not implemented")
    }
}

impl ToLogPayload for SetSucNodeIdCallback {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("success", self.success)
            .into()
    }
}
