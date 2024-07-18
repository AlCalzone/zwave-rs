use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use custom_debug_derive::Debug;
use zwave_cc::prelude::*;
use zwave_core::parse::{
    bytes::be_u8,
    combinators::{map_res, opt},
    multi::length_value,
};
use zwave_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationCommandRequest {
    pub frame_info: FrameInfo,
    pub address: CCAddress,
    // Saving the address on the CC and the command separately is a bit redundant.
    // Consider making address a getter and reading the CC field
    pub command: WithAddress<CC>,
    pub rssi: Option<RSSI>,
}

impl ApplicationCommandRequest {
    pub fn get_cc_parsing_context<'a>(
        &self,
        cmd_ctx: CommandParsingContext,
    ) -> CCParsingContext {
        CCParsingContext::builder()
            .source_node_id(self.address.source_node_id)
            .frame_addressing(self.frame_info.frame_addressing)
            .own_node_id(cmd_ctx.own_node_id)
            .security_manager(cmd_ctx.security_manager)
            .build()
    }
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
    fn parse(i: &mut Bytes, ctx: CommandParsingContext) -> ParseResult<Self> {
        let frame_info = FrameInfo::parse(i)?;
        let source_node_id = NodeId::parse(i, ctx.node_id_type)?;

        let cc_raw = length_value(be_u8, CCRaw::parse).parse(i)?;
        let cc_ctx = CCParsingContext::builder()
            .source_node_id(source_node_id)
            .frame_addressing(frame_info.frame_addressing)
            .own_node_id(ctx.own_node_id)
            .security_manager(ctx.security_manager)
            .build();
        let cc = CC::try_from_raw(cc_raw, cc_ctx)?;

        let rssi = opt(RSSI::parse).parse(i)?;

        let destination = match frame_info.frame_addressing {
            FrameAddressing::Singlecast => Destination::Singlecast(ctx.own_node_id),
            FrameAddressing::Broadcast => Destination::Broadcast,
            FrameAddressing::Multicast => Destination::Multicast(vec![ctx.own_node_id]),
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

impl SerializableWith<&CommandEncodingContext> for ApplicationCommandRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: ApplicationCommandRequest::serialize() not implemented");
    }
}

impl ToLogPayload for ApplicationCommandRequest {
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

        ret = ret.with_nested(self.command.to_log_payload());

        ret.into()
    }
}
