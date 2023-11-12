use crate::prelude::*;
use hex::ToHex;
use nom::bytes::complete::take;
use nom::combinator::{cond, opt};
use nom::number::complete::{be_u16, be_u8};
use nom::sequence::tuple;
use zwave_core::encoding::encoders::empty;
use zwave_core::{encoding, prelude::*};

use cookie_factory as cf;
use nom::{bytes::complete::tag, character::complete::none_of, combinator::map, multi::many1};

#[derive(Debug, Clone, PartialEq)]
pub struct GetProtocolVersionRequest {}

impl GetProtocolVersionRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parsable for GetProtocolVersionRequest {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for GetProtocolVersionRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl CommandRequest for GetProtocolVersionRequest {
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
pub struct GetProtocolVersionResponse {
    pub protocol_type: ProtocolType,
    pub version: Version,
    pub app_framework_build_number: Option<u16>,
    pub git_commit_hash: Option<String>,
}

impl Parsable for GetProtocolVersionResponse {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, protocol_type) = ProtocolType::parse(i)?;
        let (i, version) = map(tuple((be_u8, be_u8, be_u8)), |(major, minor, patch)| {
            Version {
                major,
                minor,
                patch,
            }
        })(i)?;
        let (i, app_framework_build_number) = opt(be_u16)(i)?;
        let (i, git_commit_hash) = map(
            cond(app_framework_build_number.is_some(), opt(take(16usize))),
            |o| {
                o.flatten().and_then(|hash: &[u8]| {
                    // An empty hash may be serialized as all zeroes
                    if !hash.iter().all(|&b| b == 0) {
                        Some(hash.encode_hex::<String>())
                    } else {
                        None
                    }
                })
            },
        )(i)?;

        Ok((
            i,
            Self {
                protocol_type,
                version,
                app_framework_build_number,
                git_commit_hash,
            },
        ))
    }
}

impl Serializable for GetProtocolVersionResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |out| todo!()
    }
}
