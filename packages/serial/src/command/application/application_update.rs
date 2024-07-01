use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::TryFromRepr;
use zwave_core::{parse::{
    bytes::{be_u32, be_u8},
    combinators::map_res,
}};
use zwave_core::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum ApplicationUpdateType {
    SmartStartHomeIdReceivedLR = 0x87, // A smart start node requests inclusion via Z-Wave Long Range
    SmartStartIncludedNodeInfoReceived = 0x86, // An included smart start node has been powered up
    SmartStartHomeIdReceived = 0x85,   // A smart start node requests inclusion
    NodeInfoReceived = 0x84,
    NodeInfoRequestDone = 0x82,
    NodeInfoRequestFailed = 0x81,
    RoutingPending = 0x80,
    NodeAdded = 0x40,   // A new node was added to the network by another controller
    NodeRemoved = 0x20, // A new node was removed from the network by another controller
    SucIdChanged = 0x10,
}

impl Parsable for ApplicationUpdateType {
    fn parse(i: &mut Bytes) -> ParseResult<Self> {
        map_res(be_u8, ApplicationUpdateType::try_from).parse(i)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationUpdateRequest {
    pub update_type: ApplicationUpdateType,
    pub payload: ApplicationUpdateRequestPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationUpdateRequestPayload {
    SucIdChanged,
    RoutingPending,

    NodeInfoReceived {
        node_id: NodeId,
        application_data: NodeInformationApplicationData,
    },
    NodeInfoRequestDone,   // TODO: Includes node_id?
    NodeInfoRequestFailed, // TODO: Includes node_id?

    NodeAdded {
        node_id: NodeId,
        application_data: NodeInformationApplicationData,
    },
    NodeRemoved {
        node_id: NodeId,
    },

    SmartStartHomeIdReceived {
        node_id: NodeId,
        nwi_home_id: u32,
        application_data: NodeInformationApplicationData,
    },
    SmartStartHomeIdReceivedLR {
        node_id: NodeId,
        nwi_home_id: u32,
        application_data: NodeInformationApplicationData,
    },
    SmartStartIncludedNodeInfoReceived,
}

impl CommandId for ApplicationUpdateRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::ApplicationUpdateRequest
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for ApplicationUpdateRequest {
    fn is_ok(&self) -> bool {
        #[allow(
            clippy::match_like_matches_macro,
            reason = "matches!() would have to be negated, harder to read"
        )]
        match self.payload {
            ApplicationUpdateRequestPayload::NodeInfoRequestFailed => false,
            _ => true,
        }
    }
}

impl CommandParsable for ApplicationUpdateRequest {
    fn parse(i: &mut Bytes, ctx: &CommandParsingContext) -> ParseResult<Self> {
        let update_type = ApplicationUpdateType::parse(i)?;
        let payload = match update_type {
            ApplicationUpdateType::SucIdChanged => ApplicationUpdateRequestPayload::SucIdChanged,
            ApplicationUpdateType::RoutingPending => {
                ApplicationUpdateRequestPayload::RoutingPending
            }

            ApplicationUpdateType::NodeInfoReceived => {
                let node_id = NodeId::parse(i, ctx.node_id_type)?;
                let application_data = NodeInformationApplicationData::parse(i)?;
                ApplicationUpdateRequestPayload::NodeInfoReceived {
                    node_id,
                    application_data,
                }
            }

            ApplicationUpdateType::NodeInfoRequestDone => {
                ApplicationUpdateRequestPayload::NodeInfoRequestDone
            }
            ApplicationUpdateType::NodeInfoRequestFailed => {
                ApplicationUpdateRequestPayload::NodeInfoRequestFailed
            }

            ApplicationUpdateType::NodeAdded => {
                let node_id = NodeId::parse(i, ctx.node_id_type)?;
                let application_data = NodeInformationApplicationData::parse(i)?;
                ApplicationUpdateRequestPayload::NodeAdded {
                    node_id,
                    application_data,
                }
            }
            ApplicationUpdateType::NodeRemoved => {
                let node_id = NodeId::parse(i, ctx.node_id_type)?;
                ApplicationUpdateRequestPayload::NodeRemoved { node_id }
            }

            ApplicationUpdateType::SmartStartHomeIdReceived => {
                let node_id = NodeId::parse(i, ctx.node_id_type)?;
                let nwi_home_id = be_u32(i)?;
                let application_data = NodeInformationApplicationData::parse(i)?;
                ApplicationUpdateRequestPayload::SmartStartHomeIdReceived {
                    node_id,
                    nwi_home_id,
                    application_data,
                }
            }
            ApplicationUpdateType::SmartStartHomeIdReceivedLR => {
                let node_id = NodeId::parse(i, ctx.node_id_type)?;
                let nwi_home_id = be_u32(i)?;
                let application_data = NodeInformationApplicationData::parse(i)?;
                ApplicationUpdateRequestPayload::SmartStartHomeIdReceivedLR {
                    node_id,
                    nwi_home_id,
                    application_data,
                }
            }
            ApplicationUpdateType::SmartStartIncludedNodeInfoReceived => {
                ApplicationUpdateRequestPayload::SmartStartIncludedNodeInfoReceived
            }
        };
        Ok(Self {
            update_type,
            payload,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for ApplicationUpdateRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: ApplicationUpdateRequest::serialize() not implemented")
    }
}

impl ToLogPayload for ApplicationUpdateRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadText::new("TODO: implement ToLogPayload for ApplicationUpdateRequest").into()
    }
}
