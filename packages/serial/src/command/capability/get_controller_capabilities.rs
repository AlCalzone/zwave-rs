use crate::prelude::*;
use ux::{u1, u3};
use zwave_core::{encoding::BitParsable, prelude::*};

use nom::{bits, complete::bool, sequence::tuple};
use zwave_core::encoding::{self, encoders::empty};

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
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for GetControllerCapabilitiesRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerCapabilitiesResponse {
    role: ControllerRole,
    started_this_network: bool,
    sis_present: bool,
    is_suc: bool,
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
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        let (i, (_reserved765, is_suc, _reserved3, sis_present, other_network, secondary)) =
            bits(tuple((u3::parse, bool, u1::parse, bool, bool, bool)))(i)?;
        Ok((
            i,
            Self {
                role: if secondary {
                    ControllerRole::Secondary
                } else {
                    ControllerRole::Primary
                },
                started_this_network: !other_network,
                sis_present,
                is_suc,
            },
        ))
    }
}

impl Serializable for GetControllerCapabilitiesResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: GetControllerCapabilitiesResponse::serialize() not implemented")
    }
}
