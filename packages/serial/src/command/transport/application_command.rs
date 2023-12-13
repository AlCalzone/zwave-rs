use crate::{prelude::*, util::hex_fmt, command::CommandId};
use zwave_core::prelude::*;


use custom_debug_derive::Debug;

use nom::{combinator::opt, multi::length_data, number::complete::be_u8};
use zwave_core::encoding::{self};

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationCommandRequest {
    frame_info: FrameInfo,
    source_node_id: NodeId,
    #[debug(with = "hex_fmt")]
    payload: Vec<u8>,
    rssi: Option<RSSI>,
}

impl CommandId for ApplicationCommandRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::ApplicationCommand
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for ApplicationCommandRequest {}

impl CommandParsable for ApplicationCommandRequest {
    fn parse<'a>(i: encoding::Input<'a>, ctx: &CommandEncodingContext) -> encoding::ParseResult<'a, Self> {
        let (i, frame_info) = FrameInfo::parse(i)?;
        let (i, source_node_id) = NodeId::parse(i, ctx.node_id_type)?;
        let (i, payload) = length_data(be_u8)(i)?;
        let (i, rssi) = opt(RSSI::parse)(i)?;

        Ok((
            i,
            Self {
                frame_info,
                source_node_id,
                payload: payload.to_vec(),
                rssi,
            },
        ))
    }
}

impl CommandSerializable for ApplicationCommandRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self, _ctx: &'a CommandEncodingContext) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        move |_out| todo!("ERROR: ApplicationCommandRequest::serialize() not implemented")
    }
}
