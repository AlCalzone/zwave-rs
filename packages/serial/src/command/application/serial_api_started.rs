use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use ux::u7;
use zwave_core::parse::{
    bits::{self, bool},
    bytes::be_u8,
    combinators::map, multi::variable_length_cc_list,
};
use zwave_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SerialApiStartedRequest {
    wake_up_reason: SerialApiWakeUpReason,
    watchdog_enabled: bool,
    generic_device_class: u8,
    specific_device_class: u8,
    is_listening: bool,
    supported_command_classes: Vec<CommandClasses>,
    controlled_command_classes: Vec<CommandClasses>,
    supports_long_range: bool,
}

impl CommandId for SerialApiStartedRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SerialApiStarted
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for SerialApiStartedRequest {}

impl CommandParsable for SerialApiStartedRequest {
    fn parse(i: &mut Bytes, _ctx: CommandParsingContext) -> ParseResult<Self> {
        let wake_up_reason = SerialApiWakeUpReason::parse(i)?;
        let watchdog_enabled = map(be_u8, |x| x == 0x01).parse(i)?;
        let (is_listening, _reserved) = bits::bits((bool, u7::parse)).parse(i)?;
        let generic_device_class = be_u8(i)?;
        let specific_device_class = be_u8(i)?;
        let (supported_command_classes, controlled_command_classes) =
            variable_length_cc_list(i)?;
        let (_reserved, supports_long_range) = bits::bits((u7::parse, bool)).parse(i)?;

        Ok(Self {
            wake_up_reason,
            watchdog_enabled,
            generic_device_class,
            specific_device_class,
            is_listening,
            supported_command_classes,
            controlled_command_classes,
            supports_long_range,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for SerialApiStartedRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: SerialApiStartedRequest::serialize() not implemented")
    }
}

impl ToLogPayload for SerialApiStartedRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("wake up reason", self.wake_up_reason.to_string())
            .with_entry("watchdog enabled", self.watchdog_enabled)
            .with_entry(
                "generic device class",
                format!("0x{:02x}", self.generic_device_class),
            )
            .with_entry(
                "specific device class",
                format!("0x{:02x}", self.specific_device_class),
            )
            .with_entry("always listening", self.is_listening)
            .with_entry("supports Long Range", self.supports_long_range)
            .into()
    }
}
