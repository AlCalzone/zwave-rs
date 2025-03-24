use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use zwave_cc::commandclass::CcOrRaw;
use zwave_cc::prelude::*;
use zwave_core::parse::multi::variable_length_bitmask_u8;
use zwave_core::parse::{
    bytes::be_u8,
    combinators::opt,
    multi::length_value,
};
use zwave_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeApplicationCommandRequest {
    pub frame_info: FrameInfo,
    pub address: CCAddress,
    // Saving the address on the CC and the command separately is a bit redundant.
    // Consider making address a getter and reading the CC field
    pub command: WithAddress<CcOrRaw>,
    pub rssi: Option<RSSI>,
}

// impl BridgeApplicationCommandRequest {
//     pub fn get_cc_parsing_context(
//         &self,
//         cmd_ctx: CommandParsingContext,
//     ) -> CCParsingContext {
//         CCParsingContext::builder()
//             .source_node_id(self.address.source_node_id)
//             .frame_addressing(self.frame_info.frame_addressing)
//             .own_node_id(cmd_ctx.own_node_id)
//             .security_manager(cmd_ctx.security_manager)
//             .build()
//     }
// }

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
    fn parse(i: &mut Bytes, ctx: CommandParsingContext) -> ParseResult<Self> {
        let frame_info = FrameInfo::parse(i)?;
        let destination_node_id = NodeId::parse(i, ctx.node_id_type)?;
        let source_node_id = NodeId::parse(i, ctx.node_id_type)?;

        // let cc_ctx = CCParsingContext::builder()
        //     .source_node_id(source_node_id)
        //     .frame_addressing(frame_info.frame_addressing)
        //     .own_node_id(ctx.own_node_id)
        //     .security_manager(ctx.security_manager)
        //     .build();
        let cc_raw = length_value(be_u8, CCRaw::parse).parse(i)?;
        // let cc = CC::try_from_raw(cc_raw, cc_ctx)?;

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

        let cc = CcOrRaw::Raw(cc_raw).with_address(address.clone());

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
        todo!("ERROR: BridgeApplicationCommandRequest::serialize() not implemented");
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

        if let CcOrRaw::CC(cc) = &self.command.as_ref() {
            ret = ret.with_nested(cc.to_log_payload());
        }

        ret.into()
    }
}
