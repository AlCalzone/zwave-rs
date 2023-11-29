use crate::prelude::*;
use ux::u7;
use zwave_core::{encoding::BitParsable, prelude::*};

use cookie_factory as cf;
use custom_debug_derive::Debug;

use nom::{
    bits, combinator::map, complete::bool, multi::length_data, number::complete::be_u8,
    sequence::tuple,
};
use zwave_core::encoding::{self};

#[derive(Debug, Clone, PartialEq)]
pub struct SerialAPIStartedRequest {
    wake_up_reason: SerialAPIWakeUpReason,
    watchdog_enabled: bool,
    generic_device_class: u8,
    specific_device_class: u8,
    is_listening: bool,
    supported_command_classes: Vec<u8>, // FIXME: Use the CommandClasses enum
    controlled_command_classes: Vec<u8>, // FIXME: Use the CommandClasses enum
    supports_long_range: bool,
}

impl CommandBase for SerialAPIStartedRequest {}

impl Parsable for SerialAPIStartedRequest {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, wake_up_reason) = SerialAPIWakeUpReason::parse(i)?;
        let (i, watchdog_enabled) = map(be_u8, |x| x == 0x01)(i)?;
        let (i, (is_listening, _reserved)) = bits(tuple((bool, u7::parse)))(i)?;
        let (i, generic_device_class) = be_u8(i)?;
        let (i, specific_device_class) = be_u8(i)?;
        let (i, supported_command_classes) = map(length_data(be_u8), |x: &[u8]| x.to_vec())(i)?; // FIXME: Parse variable-length CCs, stop at SUPPORT/CONTROL MARK
        let controlled_command_classes = Vec::new();
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

impl Serializable for SerialAPIStartedRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};
        move |out| todo!("ERROR: SerialAPIStartedRequest::serialize() not implemented")
    }
}
