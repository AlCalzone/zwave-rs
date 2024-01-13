use std::borrow::Cow;

use crate::prelude::*;
use zwave_core::prelude::*;

use zwave_core::encoding::{self};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetNodeProtocolInfoRequest {
    pub node_id: NodeId,
}

impl CommandId for GetNodeProtocolInfoRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetNodeProtocolInfo
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetNodeProtocolInfoRequest {}

impl CommandRequest for GetNodeProtocolInfoRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetNodeProtocolInfoRequest {
    fn parse<'a>(
        i: encoding::Input<'a>,
        ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, node_id) = NodeId::parse(i, ctx.node_id_type)?;
        Ok((i, Self { node_id }))
    }
}

impl CommandSerializable for GetNodeProtocolInfoRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        self.node_id.serialize(ctx.node_id_type)
    }
}

impl ToLogPayload for GetNodeProtocolInfoRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("node ID", self.node_id.to_string())
            .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetNodeProtocolInfoResponse {
    pub protocol_info: NodeInformationProtocolData,
}

impl CommandId for GetNodeProtocolInfoResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetNodeProtocolInfo
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetNodeProtocolInfoResponse {}

impl CommandParsable for GetNodeProtocolInfoResponse {
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, protocol_info) = NodeInformationProtocolData::parse(i)?;
        Ok((i, Self { protocol_info }))
    }
}

impl CommandSerializable for GetNodeProtocolInfoResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: GetNodeProtocolInfoResponse::serialize() not implemented")
    }
}

impl ToLogPayload for GetNodeProtocolInfoResponse {
    fn to_log_payload(&self) -> LogPayload {
        let info = &self.protocol_info;
        let listen: Cow<_> = match (&info.listening, &info.frequent_listening) {
            (true, _) => Cow::from("always listening"),
            (false, None) => Cow::from("sleeping"),
            (false, Some(beam)) => Cow::from(format!("frequent listening ({})", beam)),
        };

        let mut ret = LogPayloadDict::new()
            .with_entry(
                "basic device class",
                info.basic_device_type
                    .expect("basic device class should be set")
                    .to_string(),
            )
            .with_entry(
                "generic device class",
                format!("0x{:02x}", info.generic_device_class),
            );

        if let Some(specific) = info.specific_device_class {
            ret = ret.with_entry("specific device class", format!("0x{:02x}", specific))
        }

        ret = ret
            .with_entry("node type", info.node_type.to_string())
            .with_entry("listening", listen.to_string())
            .with_entry(
                "maximum data rate",
                info.supported_data_rates
                    .iter()
                    .max()
                    .unwrap_or(&DataRate::DataRate_9k6)
                    .to_string(),
            )
            .with_entry("can route", info.routing)
            .with_entry("supports beaming", info.beaming)
            .with_entry("supports security", info.supports_security)
            .with_entry("protocol version", info.protocol_version.to_string());

        ret.into()
    }
}
