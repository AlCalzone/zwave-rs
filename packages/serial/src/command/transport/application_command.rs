use crate::{prelude::*, util::hex_fmt};
use zwave_core::prelude::*;

use cookie_factory as cf;
use custom_debug_derive::Debug;

use nom::{combinator::opt, multi::length_data, number::complete::be_u8};
use zwave_core::encoding::{self};

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationCommandRequest {
    frame_info: FrameInfo,
    source_node_id: u16,
    #[debug(with = "hex_fmt")]
    payload: Vec<u8>,
    rssi: Option<RSSI>,
}

impl CommandBase for ApplicationCommandRequest {}

impl Parsable for ApplicationCommandRequest {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, frame_info) = FrameInfo::parse(i)?;
        let (i, source_node_id) = be_u8(i)?; // FIXME: This needs to depend on the controller's node ID type
        let (i, payload) = length_data(be_u8)(i)?;
        let (i, rssi) = opt(RSSI::parse)(i)?;

        Ok((
            i,
            Self {
                frame_info,
                source_node_id: source_node_id as u16,
                payload: payload.to_vec(),
                rssi,
            },
        ))
    }
}

impl Serializable for ApplicationCommandRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};
        move |out| todo!("ERROR: ApplicationCommandRequest::serialize() not implemented")
    }
}
