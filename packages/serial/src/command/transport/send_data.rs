use crate::prelude::*;
use zwave_core::prelude::*;

use cookie_factory as cf;
use derive_builder::Builder;
use nom::{bytes::complete::take, combinator::map, number::complete::be_u8};
use zwave_core::encoding;

#[derive(Default, Debug, Clone, PartialEq)]
#[derive(Builder)]
#[builder(pattern = "owned")]
#[builder(build_fn(error = "crate::error::Error"))]
pub struct SendDataRequest {
    node_id: u16,
    #[builder(setter(skip))]
    callback_id: Option<u8>,
    transmit_options: TransmitOptions,
    payload: Vec<u8>, // FIXME: This should be a CommandClass
}

impl SendDataRequest {
    pub fn builder() -> SendDataRequestBuilder {
        SendDataRequestBuilder::default()
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

impl Parsable for SendDataRequest {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, node_id) = be_u8(i)?; // FIXME: This needs to depend on the controller's node ID type
        let (i, payload_len) = be_u8(i)?;
        let (i, payload) = take(payload_len)(i)?;
        let (i, transmit_options) = TransmitOptions::parse(i)?;
        let (i, callback_id) = be_u8(i)?;

        Ok((
            i,
            Self {
                node_id: node_id as u16,
                callback_id: Some(callback_id),
                transmit_options,
                payload: payload.to_vec(),
            },
        ))
    }
}

impl Serializable for SendDataRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice, sequence::tuple};
        tuple((
            be_u8(self.node_id as u8), // FIXME: This needs to depend on the controller's node ID type
            be_u8(self.payload.len() as u8),
            slice(&self.payload), // FIXME: This must be the serialized CC
            self.transmit_options.serialize(),
            be_u8(self.callback_id.unwrap_or(0)),
        ))
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

impl Parsable for SendDataResponse {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, was_sent) = map(be_u8, |x| x > 0)(i)?;
        Ok((i, Self { was_sent }))
    }
}

impl Serializable for SendDataResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(if self.was_sent { 1 } else { 0 })
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

impl Parsable for SendDataCallback {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, callback_id) = be_u8(i)?;
        let (i, transmit_status) = TransmitStatus::parse(i)?;
        let (i, transmit_report) = TransmitReport::parse(i, transmit_status != TransmitStatus::NoAck)?;

        Ok((
            i,
            Self {
                callback_id: Some(callback_id),
                transmit_status,
                transmit_report
            },
        ))
    }
}

impl Serializable for SendDataCallback {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};
        move |out| todo!()
    }
}
