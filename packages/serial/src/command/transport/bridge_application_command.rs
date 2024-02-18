use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use custom_debug_derive::Debug;
use zwave_cc::prelude::*;
use zwave_core::parse::multi::variable_length_bitmask_u8;
use zwave_core::serialize::SerializableWith;
use zwave_core::parse::{
    bytes::be_u8,
    combinators::{map_res, opt},
    multi::length_value,
};
use zwave_core::prelude::*;

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
    fn parse(i: &mut Bytes, ctx: &CommandEncodingContext) -> ParseResult<Self> {
        let frame_info = FrameInfo::parse(i)?;
        let destination_node_id = NodeId::parse(i, ctx.node_id_type)?;
        let source_node_id = NodeId::parse(i, ctx.node_id_type)?;
        let cc = map_res(length_value(be_u8, CCRaw::parse), |raw| {
            let ctx = CCParsingContext::default();
            CC::try_from_raw(raw, &ctx)
        })
        .parse(i)?;
        let multicast_node_id_bitmask = variable_length_bitmask_u8(i, 1)?;
        let rssi = opt(RSSI::parse).parse(i)?;

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

        Ok(Self {
            frame_info,
            address,
            command: cc,
            rssi,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for BridgeApplicationCommandRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: BridgeApplicationCommandRequest::write() not implemented");
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
        let mut ret = LogPayloadDict::new();
        if let Some(rssi) = self.rssi {
            ret = ret.with_entry("RSSI", rssi.to_string())
        }

        if !infos.is_empty() {
            ret = ret.with_entry("frame info", infos.join(", "))
        }

        ret = ret
            // FIXME: log the included CC too
            .with_entry("command", "TODO: Log CC");

        ret.into()
    }
}
