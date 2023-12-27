use crate::prelude::*;
use ux::u7;
use zwave_core::{
    encoding::{parsers, BitParsable},
    prelude::*,
};

use custom_debug_derive::Debug;

use nom::{bits, combinator::map, complete::bool, number::complete::be_u8, sequence::tuple};
use zwave_core::encoding::{self};

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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, wake_up_reason) = SerialApiWakeUpReason::parse(i)?;
        let (i, watchdog_enabled) = map(be_u8, |x| x == 0x01)(i)?;
        let (i, (is_listening, _reserved)) = bits(tuple((bool, u7::parse)))(i)?;
        let (i, generic_device_class) = be_u8(i)?;
        let (i, specific_device_class) = be_u8(i)?;
        let (i, (supported_command_classes, controlled_command_classes)) =
            parsers::variable_length_cc_list(i)?;
        let (i, (_reserved, supports_long_range)) = bits(tuple((u7::parse, bool)))(i)?;

        Ok((
            i,
            Self {
                wake_up_reason,
                watchdog_enabled,
                generic_device_class,
                specific_device_class,
                is_listening,
                supported_command_classes,
                controlled_command_classes,
                supports_long_range,
            },
        ))
    }
}

impl CommandSerializable for SerialApiStartedRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: SerialApiStartedRequest::serialize() not implemented")
    }
}
