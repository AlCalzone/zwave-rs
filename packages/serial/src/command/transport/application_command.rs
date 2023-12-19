use crate::{command::CommandId, prelude::*, util::hex_fmt};
use zwave_cc::{
    commandclass::{CCParsingContext, CC},
    commandclass_raw::CCRaw,
};
use zwave_core::prelude::*;

use custom_debug_derive::Debug;

use nom::{
    combinator::{map_res, opt},
    multi::{length_data, length_value},
    number::complete::be_u8,
};
use zwave_core::encoding::{self};

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationCommandRequest {
    pub frame_info: FrameInfo,
    pub source_node_id: NodeId,
    pub command: CC,
    pub rssi: Option<RSSI>,
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, frame_info) = FrameInfo::parse(i)?;
        let (i, source_node_id) = NodeId::parse(i, ctx.node_id_type)?;
        let (i, cc) = map_res(length_value(be_u8, CCRaw::parse), |raw| {
            let ctx = CCParsingContext::default();
            CC::try_from_raw(raw, &ctx)
        })(i)?;
        let (i, rssi) = opt(RSSI::parse)(i)?;

        Ok((
            i,
            Self {
                frame_info,
                source_node_id,
                command: cc,
                rssi,
            },
        ))
    }
}

impl CommandSerializable for ApplicationCommandRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: ApplicationCommandRequest::serialize() not implemented")
    }
}
