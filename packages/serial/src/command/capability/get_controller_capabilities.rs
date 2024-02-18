use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use ux::{u1, u3};
use zwave_core::serialize::SerializableWith;
use zwave_core::parse::bits::{self, bool};
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetControllerCapabilitiesRequest {}

impl CommandId for GetControllerCapabilitiesRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetControllerCapabilities
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetControllerCapabilitiesRequest {}

impl CommandRequest for GetControllerCapabilitiesRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetControllerCapabilitiesRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CommandEncodingContext> for GetControllerCapabilitiesRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for GetControllerCapabilitiesRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerCapabilitiesResponse {
    pub role: ControllerRole,
    pub started_this_network: bool,
    pub sis_present: bool,
    pub is_suc: bool,
    // no_nodes_included: bool, // This flag is sometimes set when there are nodes in the network, so we ignore it
}

impl CommandId for GetControllerCapabilitiesResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetControllerCapabilities
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetControllerCapabilitiesResponse {}

impl CommandParsable for GetControllerCapabilitiesResponse {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> ParseResult<Self> {
        let (_reserved765, is_suc, _reserved3, sis_present, other_network, secondary) =
            bits::bits((u3::parse, bool, u1::parse, bool, bool, bool)).parse(i)?;
        Ok(Self {
            role: if secondary {
                ControllerRole::Secondary
            } else {
                ControllerRole::Primary
            },
            started_this_network: !other_network,
            sis_present,
            is_suc,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for GetControllerCapabilitiesResponse {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: GetControllerCapabilitiesResponse::write() not implemented")
    }
}

impl ToLogPayload for GetControllerCapabilitiesResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("controller role", format!("{:?}", self.role))
            .with_entry("started this network", self.started_this_network)
            .with_entry("is SUC", self.is_suc)
            .with_entry("SIS present", self.sis_present)
            .into()
    }
}
