use crate::prelude::*;
use bytes::Bytes;
use zwave_core::encoding::encoders::empty;
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetSucNodeIdRequest {}

impl CommandId for GetSucNodeIdRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetSUCNodeId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetSucNodeIdRequest {}

impl CommandRequest for GetSucNodeIdRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetSucNodeIdRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl CommandSerializable for GetSucNodeIdRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl ToLogPayload for GetSucNodeIdRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetSucNodeIdResponse {
    pub suc_node_id: Option<NodeId>,
}

impl CommandId for GetSucNodeIdResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetSUCNodeId
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetSucNodeIdResponse {}

impl CommandParsable for GetSucNodeIdResponse {
    fn parse(i: &mut Bytes, ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let suc_node_id = NodeId::parse(i, ctx.node_id_type)?;
        Ok(Self {
            suc_node_id: if suc_node_id == 0u8 {
                None
            } else {
                Some(suc_node_id)
            },
        })
    }
}

impl CommandSerializable for GetSucNodeIdResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |out| {
            self.suc_node_id
                .unwrap_or(NodeId::new(0u8))
                .serialize(ctx.node_id_type)(out)
        }
    }
}

impl ToLogPayload for GetSucNodeIdResponse {
    fn to_log_payload(&self) -> LogPayload {
        if let Some(suc_node_id) = self.suc_node_id {
            LogPayloadDict::new()
                .with_entry("SUC node ID", suc_node_id.to_string())
                .into()
        } else {
            LogPayloadText::new("no SUC").into()
        }
    }
}
