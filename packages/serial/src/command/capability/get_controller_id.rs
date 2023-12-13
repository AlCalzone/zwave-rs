use crate::prelude::*;
use zwave_core::prelude::*;

use custom_debug_derive::Debug;
use cookie_factory as cf;

use nom::{
    number::complete::{be_u32},
};
use zwave_core::encoding::{self, encoders::empty};

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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CommandSerializable for GetControllerIdRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self, _ctx: &'a CommandEncodingContext) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        // No payload
        empty()
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, home_id) = be_u32(i)?;
        let (i, own_node_id) = NodeId::parse(i, ctx.node_id_type)?;

        Ok((
            i,
            Self {
                home_id,
                own_node_id,
            },
        ))
    }
}

impl CommandSerializable for GetControllerIdResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self, _ctx: &'a CommandEncodingContext) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u32, sequence::tuple};
        // FIXME: Support serializing 16-bit node IDs
        tuple((
            be_u32(self.home_id),
            self.own_node_id.serialize(NodeIdType::NodeId8Bit),
        ))
    }
}
