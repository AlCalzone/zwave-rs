use crate::prelude::*;
use zwave_core::prelude::*;




use zwave_core::encoding::{self};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetNodeProtocolInfoRequest {
    pub node_id: NodeId,
}

impl CommandId for GetNodeProtocolInfoRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetNodeProtocolInfo
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetNodeProtocolInfoRequest {}

impl CommandRequest for GetNodeProtocolInfoRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetNodeProtocolInfoRequest {
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
        Ok((i, Self { node_id }))
    }
}

impl CommandSerializable for GetNodeProtocolInfoRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        self.node_id.serialize(ctx.node_id_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetNodeProtocolInfoResponse {
    pub protocol_info: NodeInformationProtocolData,
}

impl CommandId for GetNodeProtocolInfoResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetNodeProtocolInfo
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetNodeProtocolInfoResponse {}

impl CommandParsable for GetNodeProtocolInfoResponse {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, protocol_info) = NodeInformationProtocolData::parse(i)?;
        Ok((i, Self { protocol_info }))
    }
}

impl CommandSerializable for GetNodeProtocolInfoResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        move |_out| todo!("ERROR: GetNodeProtocolInfoResponse::serialize() not implemented")
    }
}
