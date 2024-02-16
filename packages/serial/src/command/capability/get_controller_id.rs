use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use custom_debug_derive::Debug;
use zwave_core::encoding::encoders::empty;
use zwave_core::munch::bytes::be_u32;
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
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl CommandSerializable for GetControllerIdRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
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
    fn parse(i: &mut Bytes, ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let home_id = be_u32().parse(i)?;
        let own_node_id = NodeId::parse(i, ctx.node_id_type)?;

        Ok(Self {
            home_id,
            own_node_id,
        })
    }
}

impl CommandSerializable for GetControllerIdResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u32, sequence::tuple};
        tuple((
            be_u32(self.home_id),
            self.own_node_id.serialize(ctx.node_id_type),
        ))
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
