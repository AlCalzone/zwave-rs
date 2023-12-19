use crate::prelude::*;
use zwave_cc::{
    commandclass::{CCParsingContext, CCSerializable, CC},
    commandclass_raw::CCRaw,
};
use zwave_core::prelude::*;

use cookie_factory as cf;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    multi::length_value,
    number::complete::be_u8,
    Parser,
};
use typed_builder::TypedBuilder;
use zwave_core::encoding;

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct SendDataRequest {
    #[builder(setter(into))]
    node_id: NodeId,
    command: CC,
    #[builder(setter(skip), default)]
    callback_id: Option<u8>,
    #[builder(default)]
    transmit_options: TransmitOptions,
}

impl CommandId for SendDataRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SendData
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for SendDataRequest {
    fn callback_id(&self) -> Option<u8> {
        self.callback_id
    }
}

impl CommandRequest for SendDataRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        self.callback_id.is_some()
    }

    fn needs_callback_id(&self) -> bool {
        true
    }

    fn set_callback_id(&mut self, callback_id: Option<u8>) {
        self.callback_id = callback_id;
    }
}

impl CommandParsable for SendDataRequest {
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
        let (i, cc) = map_res(length_value(be_u8, CCRaw::parse), |raw| {
            let ctx = CCParsingContext::default();
            CC::try_from_raw(raw, &ctx)
        })(i)?;
        let (i, transmit_options) = TransmitOptions::parse(i)?;
        let (i, callback_id) = be_u8(i)?;

        Ok((
            i,
            Self {
                node_id,
                callback_id: Some(callback_id),
                transmit_options,
                command: cc,
            },
        ))
    }
}

impl CommandSerializable for SendDataRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice, sequence::tuple};
        move |out| {
            // TODO: Figure out if we should handle serialization errors elsewhere
            // let error_msg = format!("Serializing command {:?} should not fail", &self.command);

            let command = self.command.clone();
            let payload = command
                .try_into_raw()
                .and_then(|raw| raw.try_to_vec())
                .expect("Serializing a CC should not fail");

            tuple((
                self.node_id.serialize(ctx.node_id_type),
                be_u8(payload.len() as u8),
                slice(payload),
                self.transmit_options.serialize(),
                be_u8(self.callback_id.unwrap_or(0)),
            ))(out)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SendDataResponse {
    was_sent: bool,
}

impl CommandBase for SendDataResponse {
    fn is_ok(&self) -> bool {
        self.was_sent
    }
}

impl CommandId for SendDataResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SendData
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandParsable for SendDataResponse {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, was_sent) = map(be_u8, |x| x > 0)(i)?;
        Ok((i, Self { was_sent }))
    }
}

impl CommandSerializable for SendDataResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(if self.was_sent { 0x01 } else { 0x00 })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SendDataCallback {
    callback_id: Option<u8>,
    transmit_status: TransmitStatus,
    transmit_report: TransmitReport,
}

impl CommandBase for SendDataCallback {
    fn is_ok(&self) -> bool {
        self.transmit_status == TransmitStatus::Ok
    }

    fn callback_id(&self) -> Option<u8> {
        self.callback_id
    }
}

impl CommandId for SendDataCallback {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SendData
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandParsable for SendDataCallback {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, callback_id) = be_u8(i)?;
        let (i, transmit_status) = TransmitStatus::parse(i)?;
        let (i, transmit_report) =
            TransmitReport::parse(i, transmit_status != TransmitStatus::NoAck)?;

        Ok((
            i,
            Self {
                callback_id: Some(callback_id),
                transmit_status,
                transmit_report,
            },
        ))
    }
}

impl CommandSerializable for SendDataCallback {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!()
    }
}
