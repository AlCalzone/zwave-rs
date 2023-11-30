use crate::{command::CommandId, prelude::*};
use zwave_core::prelude::*;

use cookie_factory as cf;
use nom::{bytes::complete::tag, character::complete::none_of, combinator::map, multi::many1};
use zwave_core::encoding::{self, encoders::empty};

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

impl Parsable for GetControllerVersionRequest {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for GetControllerVersionRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerVersionResponse {
    library_type: ZWaveLibraryType,
    library_version: String,
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

impl Parsable for GetControllerVersionResponse {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, version) = map(many1(none_of("\0")), |v| v.into_iter().collect::<String>())(i)?;
        let (i, _) = tag("\0")(i)?;
        let (i, library_type) = ZWaveLibraryType::parse(i)?;

        Ok((
            i,
            Self {
                library_type,
                library_version: version,
            },
        ))
    }
}

impl Serializable for GetControllerVersionResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::string, sequence::tuple};
        tuple((
            string(&self.library_version),
            be_u8(0),
            self.library_type.serialize(),
        ))
    }
}
