use crate::prelude::*;
use zwave_core::prelude::*;

use nom::combinator::opt;
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CommandSerializable for GetBackgroundRssiRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl ToLogPayload for GetBackgroundRssiRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, rssi0) = RSSI::parse(i)?;
        let (i, rssi1) = RSSI::parse(i)?;
        let (i, rssi2) = opt(RSSI::parse)(i)?;
        Ok((
            i,
            Self {
                rssi_channel_0: rssi0,
                rssi_channel_1: rssi1,
                rssi_channel_2: rssi2,
            },
        ))
    }
}

impl CommandSerializable for GetBackgroundRssiResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: GetBackgroundRssiResponse::serialize() not implemented")
    }
}

impl ToLogPayload for GetBackgroundRssiResponse {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new()
            .with_entry("channel 0", self.rssi_channel_0.to_string())
            .with_entry("channel 1", self.rssi_channel_1.to_string());
        if let Some(rssi) = self.rssi_channel_2 {
            ret = ret.with_entry("channel 2", rssi.to_string());
        }

        ret.into()
    }
}
