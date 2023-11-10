use crate::prelude::*;
use zwave_core::prelude::*;

use cookie_factory as cf;
use nom::{bytes::complete::tag, character::complete::none_of, combinator::map, multi::many1};
use zwave_core::encoding::{self, empty};

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerVersionRequest {}

impl GetControllerVersionRequest {
    pub fn new() -> Self {
        Self {}
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

impl CommandRequest for GetControllerVersionRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &Command) -> bool {
        response.command_type() == CommandType::Response
            && response.function_type() == self.function_type()
    }

    fn expects_callback(&self) -> bool {
        false
    }

    fn test_callback(&self, _callback: &Command) -> bool {
        false
    }

    fn callback_id(&self) -> Option<u8> {
        return None;
    }

    fn set_callback_id(&mut self, _callback_id: Option<u8>) {
        // No callback
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetControllerVersionResponse {
    library_type: ZWaveLibraryType,
    library_version: String,
}

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
