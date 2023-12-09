use crate::{prelude::*, util::hex_fmt, command::CommandId};
use zwave_core::{prelude::*, encoding::parsers::variable_length_bitmask_u8};


use custom_debug_derive::Debug;

use nom::{combinator::opt, multi::length_data, number::complete::be_u8};
use zwave_core::encoding::{self};

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeApplicationCommandRequest {
    frame_info: FrameInfo,
    destination_node_id: u16,
    source_node_id: u16,
    #[debug(with = "hex_fmt")]
    payload: Vec<u8>,
    multicast_node_ids: Vec<u16>, // FIXME: bitvec?
    rssi: Option<RSSI>,
}

impl CommandId for BridgeApplicationCommandRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::BridgeApplicationCommand
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for BridgeApplicationCommandRequest {}

impl CommandParsable for BridgeApplicationCommandRequest {
    fn parse(i: encoding::Input, _ctx: CommandParseContext) -> encoding::ParseResult<Self> {
        let (i, frame_info) = FrameInfo::parse(i)?;
        let (i, destination_node_id) = be_u8(i)?; // FIXME: This needs to depend on the controller's node ID type
        let (i, source_node_id) = be_u8(i)?; // FIXME: This needs to depend on the controller's node ID type
        let (i, payload) = length_data(be_u8)(i)?;
        let (i, multicast_node_id_bitmask) = variable_length_bitmask_u8(i, 1)?;
        let (i, rssi) = opt(RSSI::parse)(i)?;

        let multicast_node_ids = multicast_node_id_bitmask
            .iter()
            .map(|x| (*x) as u16)
            .collect();

        Ok((
            i,
            Self {
                frame_info,
                destination_node_id: destination_node_id as u16,
                source_node_id: source_node_id as u16,
                payload: payload.to_vec(),
                multicast_node_ids,
                rssi,
            },
        ))
    }
}

impl Serializable for BridgeApplicationCommandRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        
        move |_out| todo!("ERROR: BridgeApplicationCommandRequest::serialize() not implemented")
    }
}
