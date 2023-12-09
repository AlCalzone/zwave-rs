use crate::prelude::*;
use zwave_core::prelude::*;

use cookie_factory as cf;
use derive_builder::Builder;
use nom::{combinator::map, number::complete::be_u8};
use zwave_core::encoding::{self};

#[derive(Default, Debug, Clone, PartialEq, Builder)]
#[builder(pattern = "owned")]
#[builder(build_fn(error = "crate::error::Error"))]
pub struct SetRfReceiveModeRequest {
    // Whether the Z-Wave module's RF receiver should be enabled
    enabled: bool,
}

impl SetRfReceiveModeRequest {
    pub fn builder() -> SetRfReceiveModeRequestBuilder {
        SetRfReceiveModeRequestBuilder::default()
    }
}

impl CommandId for SetRfReceiveModeRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetRFReceiveMode
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for SetRfReceiveModeRequest {}

impl CommandRequest for SetRfReceiveModeRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for SetRfReceiveModeRequest {
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        eprintln!("ERROR: SetRfReceiveModeRequest::parse() not implemented");
        Ok((i, Self::default()))
    }
}

impl Serializable for SetRfReceiveModeRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(if self.enabled { 1 } else { 0 })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetRfReceiveModeResponse {
    success: bool,
}

impl CommandId for SetRfReceiveModeResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SetRFReceiveMode
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for SetRfReceiveModeResponse {
    fn is_ok(&self) -> bool {
        self.success
    }
}

impl CommandParsable for SetRfReceiveModeResponse {
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        let (i, success) = map(be_u8, |x| x > 0)(i)?;
        Ok((i, Self { success }))
    }
}

impl Serializable for SetRfReceiveModeResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: SetRfReceiveModeResponse::serialize() not implemented")
    }
}
