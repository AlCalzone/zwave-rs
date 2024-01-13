use crate::prelude::*;
use proc_macros::TryFromRepr;
use nom::{
    combinator::map_res,
    number::complete::{be_u32, be_u8},
};
use zwave_core::encoding::{self};
use zwave_core::{encoding::NomTryFromPrimitive, prelude::*};

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

impl NomTryFromPrimitive for ApplicationUpdateType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown ApplicationUpdateType: {:#04x}", repr)
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, update_type) = map_res(be_u8, ApplicationUpdateType::try_from_primitive)(i)?;
        let (i, payload) = match update_type {
            ApplicationUpdateType::SucIdChanged => {
                (i, ApplicationUpdateRequestPayload::SucIdChanged)
            }
            ApplicationUpdateType::RoutingPending => {
                (i, ApplicationUpdateRequestPayload::RoutingPending)
            }

            ApplicationUpdateType::NodeInfoReceived => {
                let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
                let (i, application_data) = NodeInformationApplicationData::parse(i)?;
                (
                    i,
                    ApplicationUpdateRequestPayload::NodeInfoReceived {
                        node_id,
                        application_data,
                    },
                )
            }

            ApplicationUpdateType::NodeInfoRequestDone => {
                (i, ApplicationUpdateRequestPayload::NodeInfoRequestDone)
            }
            ApplicationUpdateType::NodeInfoRequestFailed => {
                (i, ApplicationUpdateRequestPayload::NodeInfoRequestFailed)
            }

            ApplicationUpdateType::NodeAdded => {
                let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
                let (i, application_data) = NodeInformationApplicationData::parse(i)?;
                (
                    i,
                    ApplicationUpdateRequestPayload::NodeAdded {
                        node_id,
                        application_data,
                    },
                )
            }
            ApplicationUpdateType::NodeRemoved => {
                let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
                (i, ApplicationUpdateRequestPayload::NodeRemoved { node_id })
            }

            ApplicationUpdateType::SmartStartHomeIdReceived => {
                let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
                let (i, nwi_home_id) = be_u32(i)?;
                let (i, application_data) = NodeInformationApplicationData::parse(i)?;
                (
                    i,
                    ApplicationUpdateRequestPayload::SmartStartHomeIdReceived {
                        node_id,
                        nwi_home_id,
                        application_data,
                    },
                )
            }
            ApplicationUpdateType::SmartStartHomeIdReceivedLR => {
                let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
                let (i, nwi_home_id) = be_u32(i)?;
                let (i, application_data) = NodeInformationApplicationData::parse(i)?;
                (
                    i,
                    ApplicationUpdateRequestPayload::SmartStartHomeIdReceivedLR {
                        node_id,
                        nwi_home_id,
                        application_data,
                    },
                )
            }
            ApplicationUpdateType::SmartStartIncludedNodeInfoReceived => (
                i,
                ApplicationUpdateRequestPayload::SmartStartIncludedNodeInfoReceived,
            ),
        };
        Ok((
            i,
            Self {
                update_type,
                payload,
            },
        ))
    }
}

impl CommandSerializable for ApplicationUpdateRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: ApplicationUpdateResponse::serialize() not implemented")
    }
}

impl ToLogPayload for ApplicationUpdateRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadText::new("TODO: implement ToLogPayload for ApplicationUpdateRequest").into()
    }
}
