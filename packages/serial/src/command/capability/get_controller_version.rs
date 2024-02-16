use crate::{command::CommandId, prelude::*};
use bytes::Bytes;
use cookie_factory as cf;
use zwave_core::encoding::encoders::empty;
use zwave_core::munch::{
    bytes::complete::{literal, take_while1},
    combinators::map,
};
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetControllerVersionRequest {}

impl CommandId for GetControllerVersionRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetControllerVersion
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetControllerVersionRequest {}

impl CommandRequest for GetControllerVersionRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetControllerVersionRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl CommandSerializable for GetControllerVersionRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl ToLogPayload for GetControllerVersionRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerVersionResponse {
    pub library_type: ZWaveLibraryType,
    pub library_version: String,
}

impl CommandId for GetControllerVersionResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetControllerVersion
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetControllerVersionResponse {}

impl CommandParsable for GetControllerVersionResponse {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let version = map(take_while1(|b| b != 0), |b| {
            String::from_utf8_lossy(&b).to_string()
        })
        .parse(i)?;
        let _ = literal(0).parse(i)?;
        let library_type = ZWaveLibraryType::parse(i)?;

        Ok(Self {
            library_type,
            library_version: version,
        })
    }
}

impl CommandSerializable for GetControllerVersionResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::string, sequence::tuple};
        tuple((
            string(&self.library_version),
            be_u8(0),
            self.library_type.serialize(),
        ))
    }
}

impl ToLogPayload for GetControllerVersionResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("library type", self.library_type.to_string())
            .with_entry("library version", self.library_version.to_string())
            .into()
    }
}
