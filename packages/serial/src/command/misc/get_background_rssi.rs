use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use zwave_core::parse::combinators::opt;
use zwave_core::prelude::*;

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
    fn parse(_i: &mut Bytes, _ctx: CommandParsingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CommandEncodingContext> for GetBackgroundRssiRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        // No payload
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
    fn parse(i: &mut Bytes, _ctx: CommandParsingContext) -> ParseResult<Self> {
        let rssi0 = RSSI::parse(i)?;
        let rssi1 = RSSI::parse(i)?;
        let rssi2 = opt(RSSI::parse).parse(i)?;
        Ok(Self {
            rssi_channel_0: rssi0,
            rssi_channel_1: rssi1,
            rssi_channel_2: rssi2,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for GetBackgroundRssiResponse {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: GetBackgroundRssiResponse::serialize() not implemented");
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
