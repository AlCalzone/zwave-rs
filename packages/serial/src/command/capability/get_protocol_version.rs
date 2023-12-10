use crate::prelude::*;
use hex::ToHex;
use nom::bytes::complete::take;
use nom::combinator::{cond, opt};
use nom::number::complete::{be_u16, be_u8};
use nom::sequence::tuple;
use zwave_core::encoding::encoders::empty;
use zwave_core::{encoding, prelude::*};

use nom::combinator::map;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetProtocolVersionRequest {}

impl CommandId for GetProtocolVersionRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetProtocolVersion
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetProtocolVersionRequest {}

impl CommandRequest for GetProtocolVersionRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetProtocolVersionRequest {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CommandParseContext) -> encoding::ParseResult<'a, Self> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GetProtocolVersionResponse {
    pub protocol_type: ProtocolType,
    pub version: Version,
    pub app_framework_build_number: Option<u16>,
    pub git_commit_hash: Option<String>,
}

impl CommandId for GetProtocolVersionResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetProtocolVersion
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetProtocolVersionResponse {}

impl CommandParsable for GetProtocolVersionResponse {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CommandParseContext) -> encoding::ParseResult<'a, Self> {
        let (i, protocol_type) = ProtocolType::parse(i)?;
        let (i, version) = map(tuple((be_u8, be_u8, be_u8)), |(major, minor, patch)| {
            Version {
                major,
                minor,
                patch: Some(patch),
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
        move |_out| todo!()
    }
}
