use crate::prelude::*;
use zwave_cc::prelude::*;
use zwave_core::encoding::{self, parsers::variable_length_bitmask_u8};
use zwave_core::prelude::*;

use custom_debug_derive::Debug;

use nom::{
    combinator::{map_res, opt},
    multi::length_value,
    number::complete::be_u8,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeApplicationCommandRequest {
    pub frame_info: FrameInfo,
    pub address: CCAddress,
    // Saving the address on the CC and the command separately is a bit redundant.
    // Consider making address a getter and reading the CC field
    pub command: WithAddress<CC>,
    pub rssi: Option<RSSI>,
}

impl CommandId for BridgeApplicationCommandRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::BridgeApplicationCommand
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for BridgeApplicationCommandRequest {}

impl CommandParsable for BridgeApplicationCommandRequest {
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, frame_info) = FrameInfo::parse(i)?;
        let (i, destination_node_id) = NodeId::parse(i, ctx.node_id_type)?;
        let (i, source_node_id) = NodeId::parse(i, ctx.node_id_type)?;
        let (i, cc) = map_res(length_value(be_u8, CCRaw::parse), |raw| {
            let ctx = CCParsingContext::default();
            CC::try_from_raw(raw, &ctx)
        })(i)?;
        let (i, multicast_node_id_bitmask) = variable_length_bitmask_u8(i, 1)?;
        let (i, rssi) = opt(RSSI::parse)(i)?;

        let multicast_node_ids = multicast_node_id_bitmask
            .iter()
            .map(|x| NodeId::new(*x))
            .collect();

        let destination = match frame_info.frame_addressing {
            FrameAddressing::Singlecast => Destination::Singlecast(destination_node_id),
            FrameAddressing::Broadcast => Destination::Broadcast,
            FrameAddressing::Multicast => Destination::Multicast(multicast_node_ids),
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

impl CommandSerializable for BridgeApplicationCommandRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: BridgeApplicationCommandRequest::serialize() not implemented")
    }
}

impl ToLogPayload for BridgeApplicationCommandRequest {
    fn to_log_payload(&self) -> LogPayload {
        let mut infos: Vec<String> = Vec::new();
        match self.frame_info.frame_addressing {
            FrameAddressing::Singlecast => {}
            FrameAddressing::Broadcast => infos.push("broadcast".to_string()),
            FrameAddressing::Multicast => infos.push("multicast".to_string()),
        }
        if self.frame_info.explorer_frame {
            infos.push("explorer frame".to_string());
        }
        if self.frame_info.low_power {
            infos.push("low power".to_string());
        }
        let mut ret = LogPayloadDict::new()
            .with_entry("frame info", infos.join(", "))
            // FIXME: log the included CC too
            .with_entry("command", "TODO: Log CC");

        if let Some(rssi) = self.rssi {
            ret = ret.with_entry("RSSI", rssi.to_string())
        }

        ret.into()
    }
}
