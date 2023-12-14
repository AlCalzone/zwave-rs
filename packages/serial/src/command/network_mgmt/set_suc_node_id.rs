use crate::prelude::*;
use zwave_core::prelude::*;

use cookie_factory as cf;
use derive_builder::Builder;
use nom::{
    combinator::map,
    number::complete::be_u8,
};
use zwave_core::encoding::{self, parser_not_implemented};

#[derive(Default, Debug, Clone, PartialEq, Builder)]
#[builder(pattern = "owned")]
#[builder(build_fn(error = "crate::error::Error"))]
pub struct SetSucNodeIdRequest {
    suc_node_id: NodeId,
    enable_suc: bool,
    enable_sis: bool,
    #[builder(setter(skip))]
    callback_id: Option<u8>,
    transmit_options: TransmitOptions,
}

impl SetSucNodeIdRequest {
    pub fn builder() -> SetSucNodeIdRequestBuilder {
        SetSucNodeIdRequestBuilder::default()
    }
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
        self.suc_node_id == NodeId::new(1u8) // FIXME: This must be compared with our OWN node ID
    }

    fn needs_callback_id(&self) -> bool {
        true
    }

    fn set_callback_id(&mut self, callback_id: Option<u8>) {
        self.callback_id = callback_id;
    }
}

impl CommandParsable for SetSucNodeIdRequest {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        return parser_not_implemented(i, "ERROR: SetSucNodeIdRequest::parse() not implemented");
        // Ok((i, Self {}))
    }
}

impl CommandSerializable for SetSucNodeIdRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};
        tuple((
            self.suc_node_id.serialize(ctx.node_id_type),
            be_u8(if self.enable_suc { 0x01 } else { 0x00 }),
            self.transmit_options.serialize(),
            be_u8(if self.enable_sis { 0x01 } else { 0x00 }),
            be_u8(self.callback_id.unwrap_or(0)),
        ))
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, was_executed) = map(be_u8, |x| x > 0)(i)?;
        Ok((i, Self { was_executed }))
    }
}

impl CommandSerializable for SetSucNodeIdResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(if self.was_executed { 0x01 } else { 0x00 })
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, callback_id) = be_u8(i)?;
        let (i, status) = be_u8(i)?;

        // Status is either 0x05 (success) or 0x06 (failure)

        Ok((
            i,
            Self {
                callback_id: Some(callback_id),
                success: status == 0x05,
            },
        ))
    }
}

impl CommandSerializable for SetSucNodeIdCallback {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        move |_out| todo!("ERROR: SetSucNodeIdCallback::serialize() not implemented")
    }
}
