use crate::prelude::*;
use zwave_core::prelude::*;

use cookie_factory as cf;
use derive_builder::Builder;
use nom::{
    bytes::complete::tag,
    character::complete::none_of,
    combinator::map,
    multi::many1,
    number::complete::{be_u32, be_u8},
};
use zwave_core::encoding::{self, encoders::empty, parser_not_implemented};

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
        ctx: &CommandParseContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for GetControllerIdRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};
        // No payload
        empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerIdResponse {
    pub home_id: u32,
    pub own_node_id: u16,
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
        ctx: &CommandParseContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, home_id) = be_u32(i)?;
        // FIXME: Support parsing 16-bit node IDs
        let (i, own_node_id) = be_u8(i)?;

        Ok((
            i,
            Self {
                home_id,
                own_node_id: own_node_id as u16,
            },
        ))
    }
}

impl Serializable for GetControllerIdResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u32, bytes::be_u8, sequence::tuple};
        tuple((be_u32(self.home_id), be_u8(self.own_node_id as u8)))
    }
}
