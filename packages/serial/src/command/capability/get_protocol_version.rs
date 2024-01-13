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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CommandSerializable for GetProtocolVersionRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl ToLogPayload for GetProtocolVersionRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
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
        MessageOrigin::Controller
    }
}

impl CommandBase for GetProtocolVersionResponse {}

impl CommandParsable for GetProtocolVersionResponse {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
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

impl CommandSerializable for GetProtocolVersionResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!()
    }
}

impl ToLogPayload for GetProtocolVersionResponse {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new()
            .with_entry("protocol type", format!("{:?}", self.protocol_type))
            .with_entry("version", format!("{}", self.version));
        if let Some(app_framework_build_number) = self.app_framework_build_number {
            ret = ret.with_entry(
                "app framework build number",
                app_framework_build_number,
            )
        }
        if let Some(git_commit_hash) = &self.git_commit_hash {
            ret = ret.with_entry("git commit hash", git_commit_hash.to_string())
        }
        ret.into()
    }
}
