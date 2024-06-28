use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use typed_builder::TypedBuilder;
use zwave_cc::prelude::*;
use zwave_core::parse::{
    bytes::be_u8,
    combinators::{map, map_res},
    multi::length_value,
};
use zwave_core::prelude::*;
use zwave_core::serialize;

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct SendDataRequest {
    #[builder(setter(into))]
    pub node_id: NodeId,
    pub command: CC,
    #[builder(setter(skip), default)]
    pub callback_id: Option<u8>,
    #[builder(default)]
    pub transmit_options: TransmitOptions,
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
    fn parse(i: &mut Bytes, ctx: &mut CommandParsingContext) -> ParseResult<Self> {
        let node_id = NodeId::parse(i, ctx.node_id_type)?;
        let cc = map_res(length_value(be_u8, CCRaw::parse), |raw| {
            let mut ctx = CCParsingContext::builder()
                .security_manager(ctx.security_manager.clone())
                .build();
            CC::try_from_raw(raw, &mut ctx)
        })
        .parse(i)?;
        let transmit_options = TransmitOptions::parse(i)?;
        let callback_id = be_u8(i)?;

        Ok(Self {
            node_id,
            callback_id: Some(callback_id),
            transmit_options,
            command: cc,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for SendDataRequest {
    fn serialize(&self, output: &mut BytesMut, ctx: &CommandEncodingContext) {
        use serialize::{bytes::be_u8, bytes::slice};

        // TODO: Figure out if we should handle serialization errors elsewhere
        // let error_msg = format!("Serializing command {:?} should not fail", &self.command);

        let command = self.command.clone();
        let ccctx = CCEncodingContext::builder()
            .own_node_id(ctx.own_node_id)
            .node_id(self.node_id)
            .build();
        let payload = command.as_raw(&ccctx).as_bytes();

        self.node_id.serialize(output, ctx.node_id_type);
        be_u8(payload.len() as u8).serialize(output);
        slice(&payload).serialize(output);
        self.transmit_options.serialize(output);
        be_u8(self.callback_id.unwrap_or(0)).serialize(output);
    }
}

impl ToLogPayload for SendDataRequest {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret =
            LogPayloadDict::new().with_entry("transmit options", self.transmit_options.to_string());
        if let Some(callback_id) = self.callback_id {
            ret = ret.with_entry("callback ID", callback_id);
        }

        ret = ret.with_nested(self.command.to_log_payload());

        ret.into()
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
    fn parse(i: &mut Bytes, _ctx: &mut CommandParsingContext) -> ParseResult<Self> {
        let was_sent = map(be_u8, |x| x > 0).parse(i)?;
        Ok(Self { was_sent })
    }
}

impl SerializableWith<&CommandEncodingContext> for SendDataResponse {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        use serialize::bytes::be_u8;
        be_u8(if self.was_sent { 0x01 } else { 0x00 }).serialize(output);
    }
}

impl ToLogPayload for SendDataResponse {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("was sent", self.was_sent)
            .into()
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
    fn parse(i: &mut Bytes, _ctx: &mut CommandParsingContext) -> ParseResult<Self> {
        let callback_id = be_u8(i)?;
        let transmit_status = TransmitStatus::parse(i)?;
        let transmit_report = TransmitReport::parse(i, transmit_status != TransmitStatus::NoAck)?;

        Ok(Self {
            callback_id: Some(callback_id),
            transmit_status,
            transmit_report,
        })
    }
}

impl SerializableWith<&CommandEncodingContext> for SendDataCallback {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: SendDataCallback::serialize() not implemented")
    }
}

impl ToLogPayload for SendDataCallback {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new();
        if let Some(callback_id) = self.callback_id {
            ret = ret.with_entry("callback ID", callback_id);
        }

        ret = ret
            .with_entry(
                "transmit status",
                format!(
                    "{:?}, took {} ms",
                    self.transmit_status,
                    self.transmit_report.tx_ticks * 10
                ),
            )
            .extend(self.transmit_report.to_log_dict());

        ret.into()
    }
}
