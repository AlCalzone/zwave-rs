use crate::prelude::*;
use zwave_cc::prelude::*;
use zwave_core::encoding;
use zwave_core::prelude::*;

use custom_debug_derive::Debug;

use nom::{
    combinator::{map_res, opt},
    multi::length_value,
    number::complete::be_u8,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationCommandRequest {
    pub frame_info: FrameInfo,
    pub address: CCAddress,
    // Saving the address on the CC and the command separately is a bit redundant.
    // Consider making address a getter and reading the CC field
    pub command: WithAddress<CC>,
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

        // FIXME: Figure out the correct node ID
        let own_node_id = NodeId::new(1u8);
        let destination = match frame_info.frame_addressing {
            FrameAddressing::Singlecast => Destination::Singlecast(own_node_id),
            FrameAddressing::Broadcast => Destination::Broadcast,
            FrameAddressing::Multicast => Destination::Multicast(vec![own_node_id]),
        };
        let address = CCAddress {
            source_node_id,
            destination,
            endpoint_index: EndpointIndex::Root, // We don't know yet
        };

        let cc = cc.with_address(address.clone());

        Ok((
            i,
            Self {
                frame_info,
                address,
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
