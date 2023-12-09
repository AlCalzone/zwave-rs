use crate::prelude::*;
use zwave_core::prelude::*;



use nom::{
    combinator::{opt},
};
use zwave_core::encoding::{self, encoders::empty};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetBackgroundRssiRequest {}

impl CommandId for GetBackgroundRssiRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetBackgroundRSSI
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetBackgroundRssiRequest {}

impl CommandRequest for GetBackgroundRssiRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetBackgroundRssiRequest {
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for GetBackgroundRssiRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        // No payload
        empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetBackgroundRssiResponse {
    rssi_channel_0: RSSI,
    rssi_channel_1: RSSI,
    rssi_channel_2: Option<RSSI>,
}

impl CommandId for GetBackgroundRssiResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetBackgroundRSSI
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetBackgroundRssiResponse {}

impl CommandParsable for GetBackgroundRssiResponse {
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        let (i, rssi0) = RSSI::parse(i)?;
        let (i, rssi1) = RSSI::parse(i)?;
        let (i, rssi2) = opt(RSSI::parse)(i)?;
        eprintln!("ERROR: GetBackgroundRssiResponse::parse() not implemented");
        Ok((i, Self {
            rssi_channel_0: rssi0,
            rssi_channel_1: rssi1,
            rssi_channel_2: rssi2,
        }))
    }
}

impl Serializable for GetBackgroundRssiResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        move |_out| todo!("ERROR: GetBackgroundRssiResponse::serialize() not implemented")
    }
}
