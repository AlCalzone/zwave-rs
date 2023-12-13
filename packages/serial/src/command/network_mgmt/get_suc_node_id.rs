use crate::prelude::*;
use zwave_core::{
    encoding::{self, encoders::empty},
    prelude::*,
};

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
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CommandSerializable for GetSucNodeIdRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, suc_node_id) = NodeId::parse(i, ctx.node_id_type)?;
        Ok((
            i,
            Self {
                suc_node_id: if suc_node_id == 0u8 {
                    None
                } else {
                    Some(suc_node_id)
                },
            },
        ))
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
