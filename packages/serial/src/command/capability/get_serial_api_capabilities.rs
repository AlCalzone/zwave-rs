use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use zwave_core::parse::multi::fixed_length_bitmask_u8;
use zwave_core::log::ToLogPayload;
use zwave_core::parse::{
    bytes::{be_u16, be_u8},
    combinators::map,
};
use zwave_core::prelude::*;

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
    fn parse(_i: &mut Bytes, _ctx: CommandParsingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CommandEncodingContext> for GetSerialApiCapabilitiesRequest {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for GetSerialApiCapabilitiesRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetSerialApiCapabilitiesResponse {
    pub manufacturer_id: Id16,
    pub product_type: Id16,
    pub product_id: Id16,
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
    fn parse(i: &mut Bytes, _ctx: CommandParsingContext) -> ParseResult<Self> {
        let firmware_version = map((be_u8, be_u8), |(major, minor)| Version {
            major,
            minor,
            patch: None,
        })
        .parse(i)?;
        let manufacturer_id = be_u16(i)?;
        let product_type = be_u16(i)?;
        let product_id = be_u16(i)?;
        let supported_function_types = fixed_length_bitmask_u8(i, 1, NUM_FUNCTION_BYTES)?;
        let supported_function_types = supported_function_types
            .iter()
            .filter_map(|f| FunctionType::try_from(*f).map_or_else(|_| None, Some))
            .collect::<Vec<_>>();

        Ok(Self {
            firmware_version,
            manufacturer_id: manufacturer_id.into(),
            product_type: product_type.into(),
            product_id: product_id.into(),
            supported_function_types,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for GetSerialApiCapabilitiesResponse {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: GetSerialApiCapabilitiesResponse::serialize() not implemented");
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
