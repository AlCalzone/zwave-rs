use crate::prelude::*;
use zwave_core::{encoding::parsers::fixed_length_bitmask_u8, log::ToLogPayload, prelude::*};

use custom_debug_derive::Debug;

use nom::{
    combinator::map,
    number::complete::{be_u16, be_u8},
    sequence::tuple,
};
use zwave_core::encoding::{self, encoders::empty};

const NUM_FUNCTIONS: usize = 256;
const NUM_FUNCTION_BYTES: usize = NUM_FUNCTIONS / 8;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetSerialApiCapabilitiesRequest {}

impl GetSerialApiCapabilitiesRequest {}

impl CommandId for GetSerialApiCapabilitiesRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetSerialApiCapabilities
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetSerialApiCapabilitiesRequest {}

impl CommandRequest for GetSerialApiCapabilitiesRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetSerialApiCapabilitiesRequest {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CommandSerializable for GetSerialApiCapabilitiesRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetSerialApiCapabilitiesResponse {
    #[debug(format = "0x{:04x}")]
    pub manufacturer_id: u16,
    #[debug(format = "0x{:04x}")]
    pub product_type: u16,
    #[debug(format = "0x{:04x}")]
    pub product_id: u16,
    pub firmware_version: Version,
    pub supported_function_types: Vec<FunctionType>,
}

impl CommandId for GetSerialApiCapabilitiesResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetSerialApiCapabilities
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetSerialApiCapabilitiesResponse {}

impl CommandParsable for GetSerialApiCapabilitiesResponse {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, firmware_version) = map(tuple((be_u8, be_u8)), |(major, minor)| Version {
            major,
            minor,
            patch: None,
        })(i)?;
        let (i, manufacturer_id) = be_u16(i)?;
        let (i, product_type) = be_u16(i)?;
        let (i, product_id) = be_u16(i)?;
        let (i, supported_function_types) = fixed_length_bitmask_u8(i, 1, NUM_FUNCTION_BYTES)?;
        let supported_function_types = supported_function_types
            .iter()
            .filter_map(|f| FunctionType::try_from(*f).map_or_else(|_| None, Some))
            .collect::<Vec<_>>();

        Ok((
            i,
            Self {
                firmware_version,
                manufacturer_id,
                product_type,
                product_id,
                supported_function_types,
            },
        ))
    }
}

impl CommandSerializable for GetSerialApiCapabilitiesResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: GetSerialApiCapabilitiesResponse::serialize() not implemented")
    }
}

impl ToLogPayload for GetSerialApiCapabilitiesResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::Dict(
            LogPayloadDict::new()
                .with_entry("firmware version", self.firmware_version.to_string())
                .with_entry("manufacturer ID", format!("0x{:04x}", self.manufacturer_id))
                .with_entry("product type", format!("0x{:04x}", self.product_type))
                .with_entry("product ID", format!("0x{:04x}", self.product_id))
                .with_entry(
                    "supported function types",
                    LogPayloadList::new(
                        self.supported_function_types
                            .iter()
                            .map(|f| format!("{:?}", f).into()),
                    ),
                ),
        )
    }
}
