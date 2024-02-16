use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use custom_debug_derive::Debug;
use ux::u4;
use zwave_core::encoding::{
    encoders::{self, empty},
    parsers::variable_length_bitmask_u8,
    BitParsable, BitSerializable,
};
use zwave_core::munch::{
    bits::{self, bool},
    combinators::opt,
};
use zwave_core::prelude::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct GetSerialApiInitDataRequest {}

impl CommandId for GetSerialApiInitDataRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetSerialApiInitData
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for GetSerialApiInitDataRequest {}

impl CommandRequest for GetSerialApiInitDataRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for GetSerialApiInitDataRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl CommandSerializable for GetSerialApiInitDataRequest {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl ToLogPayload for GetSerialApiInitDataRequest {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetSerialApiInitDataResponse {
    pub api_version: ZWaveApiVersion,
    pub chip_type: Option<ChipType>,
    pub node_type: NodeType,
    pub role: ControllerRole,
    pub is_sis: bool,
    pub supports_timers: bool,
    pub node_ids: Vec<NodeId>,
}

impl CommandId for GetSerialApiInitDataResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::GetSerialApiInitData
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for GetSerialApiInitDataResponse {}

impl CommandParsable for GetSerialApiInitDataResponse {
    fn parse(i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let api_version = ZWaveApiVersion::parse(i)?;
        let (_reserved, is_sis, is_primary, supports_timers, node_type) =
            bits::bits((u4::parse, bool, bool, bool, NodeType::parse)).parse(i)?;
        let node_ids = variable_length_bitmask_u8(i, 1)?;
        let chip_type = opt(ChipType::parse).parse(i)?;
        Ok(Self {
            api_version,
            is_sis,
            role: if is_primary {
                ControllerRole::Primary
            } else {
                ControllerRole::Secondary
            },
            supports_timers,
            node_type,
            node_ids: node_ids.into_iter().map(|n| n.into()).collect(),
            chip_type,
        })
    }
}

impl CommandSerializable for GetSerialApiInitDataResponse {
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        _ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::sequence::tuple;

        move |out| {
            let node_ids: Vec<u8> = self
                .node_ids
                .iter()
                .filter_map(|n| if *n < 256u16 { Some((*n).into()) } else { None })
                .collect();

            let is_primary = self.role == ControllerRole::Primary;

            let ret = tuple((
                self.api_version.serialize(),
                encoders::bits(move |bo| {
                    let reserved = u4::new(0);
                    reserved.write(bo);
                    self.is_sis.write(bo);
                    is_primary.write(bo);
                    self.supports_timers.write(bo);
                    self.node_type.write(bo);
                }),
                encoders::bitmask_u8(&node_ids, 1),
                self.chip_type.serialize(),
            ))(out);

            ret
        }
    }
}

impl ToLogPayload for GetSerialApiInitDataResponse {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret =
            LogPayloadDict::new().with_entry("Z-Wave API version", self.api_version.to_string());
        if let Some(chip_type) = self.chip_type {
            ret = ret.with_entry("Z-Wave chip type", chip_type.to_string());
        }
        ret = ret
            .with_entry("node type", self.node_type.to_string())
            .with_entry("controller role", self.role.to_string())
            .with_entry("controller is the SIS", self.is_sis)
            .with_entry("controller supports timers", self.supports_timers)
            .with_entry(
                "nodes in the network",
                self.node_ids
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );

        ret.into()
    }
}

#[cfg(test)]
mod test {
    use crate::{command::GetSerialApiInitDataResponse, prelude::*};
    use bytes::Bytes;
    use zwave_core::prelude::*;

    #[test]
    fn test_serialize() {
        let cmd = GetSerialApiInitDataResponse {
            api_version: ZWaveApiVersion::Official(1),
            is_sis: true,
            role: ControllerRole::Primary,
            supports_timers: true,
            node_type: NodeType::Controller,
            node_ids: vec![1u8, 4, 8, 10].into_iter().map(NodeId::new).collect(),
            chip_type: Some(ChipType::EFR32xG1x),
        };
        let ctx = CommandEncodingContext::default();
        let raw: Vec<u8> = Into::<Command>::into(cmd).try_to_vec(&ctx).unwrap();
        assert_eq!(
            raw,
            vec![
                10,          // API version
                0b0000_1110, // Capabilities,
                2,           // bitmask length
                0b1000_1001, // node 1, 4, 8
                0b0000_0010, // node 10
                0x07,
                0x00, // chip type
            ]
        )
    }

    #[test]
    fn test_parse() {
        let input: Vec<u8> = vec![
            10,          // API version
            0b0000_1110, // Capabilities,
            2,           // bitmask length
            0b1000_1001, // node 1, 4, 8
            0b0000_0010, // node 10
            0x07,
            0x00, // chip type
        ];
        let mut input = Bytes::from(input);
        let expected = GetSerialApiInitDataResponse {
            api_version: ZWaveApiVersion::Official(1),
            is_sis: true,
            role: ControllerRole::Primary,
            supports_timers: true,
            node_type: NodeType::Controller,
            node_ids: vec![1u8, 4, 8, 10].into_iter().map(NodeId::new).collect(),
            chip_type: Some(ChipType::EFR32xG1x),
        };
        let actual =
            GetSerialApiInitDataResponse::parse(&mut input, &CommandEncodingContext::default())
                .unwrap();
        assert_eq!(actual, expected)
    }
}
