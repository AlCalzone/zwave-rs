use crate::prelude::*;
use zwave_core::{
    encoding::{encoders, BitSerializable},
    prelude::*,
};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use nom::{bits, bits::complete::bool, combinator::opt, sequence::tuple};
use ux::u4;
use zwave_core::encoding::{
    self, encoders::empty, parsers::variable_length_bitmask_u8, BitParsable,
};

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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
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

#[derive(Debug, Clone, PartialEq)]
pub struct GetSerialApiInitDataResponse {
    pub api_version: ZWaveApiVersion,
    pub is_sis: bool,
    pub is_primary: bool,
    pub supports_timers: bool,
    pub node_type: NodeType,
    pub node_ids: Vec<NodeId>,
    pub chip_type: Option<ChipType>,
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
    fn parse<'a>(
        i: encoding::Input<'a>,
        _ctx: &CommandEncodingContext,
    ) -> encoding::ParseResult<'a, Self> {
        let (i, api_version) = ZWaveApiVersion::parse(i)?;
        let (i, (_reserved, is_sis, is_primary, supports_timers, node_type)) =
            bits(tuple((u4::parse, bool, bool, bool, NodeType::parse)))(i)?;
        let (i, node_ids) = variable_length_bitmask_u8(i, 1)?;
        let (i, chip_type) = opt(ChipType::parse)(i)?;
        Ok((
            i,
            Self {
                api_version,
                is_sis,
                is_primary,
                supports_timers,
                node_type,
                node_ids: node_ids.into_iter().map(|n| n.into()).collect(),
                chip_type,
            },
        ))
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

            let ret = tuple((
                self.api_version.serialize(),
                encoders::bits(move |bo| {
                    let reserved = u4::new(0);
                    reserved.write(bo);
                    self.is_sis.write(bo);
                    self.is_primary.write(bo);
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

#[cfg(test)]
mod test {
    use crate::{command::GetSerialApiInitDataResponse, prelude::*};
    use zwave_core::definitions::{ChipType, NodeId, NodeType, ZWaveApiVersion};

    #[test]
    fn test_serialize() {
        let cmd = GetSerialApiInitDataResponse {
            api_version: ZWaveApiVersion::Official(1),
            is_sis: true,
            is_primary: true,
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
        let expected = GetSerialApiInitDataResponse {
            api_version: ZWaveApiVersion::Official(1),
            is_sis: true,
            is_primary: true,
            supports_timers: true,
            node_type: NodeType::Controller,
            node_ids: vec![1u8, 4, 8, 10].into_iter().map(NodeId::new).collect(),
            chip_type: Some(ChipType::EFR32xG1x),
        };
        let actual = GetSerialApiInitDataResponse::try_from_slice(
            &input,
            &CommandEncodingContext::default(),
        )
        .unwrap();
        assert_eq!(actual, expected)
    }
}
