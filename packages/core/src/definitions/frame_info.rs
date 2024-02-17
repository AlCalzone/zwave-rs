use crate::encoding;
use crate::encoding::WriteLastNBits;
use crate::munch::{
    bits::{self, bool},
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use custom_debug_derive::Debug;
use proc_macros::TryFromRepr;
use ux::u1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum FrameAddressing {
    Singlecast = 0b00,
    Broadcast = 0b01,
    Multicast = 0b10,
}

impl BitParsable for FrameAddressing {
    fn parse(i: &mut (Bytes, usize)) -> crate::munch::ParseResult<Self> {
        context(
            "FrameType",
            map_res(bits::take(2usize), |x: u8| FrameAddressing::try_from(x)),
        )
        .parse(i)
    }
}

impl BitSerializable for FrameAddressing {
    fn write(&self, b: &mut encoding::BitOutput) {
        b.write_last_n_bits((*self) as u8, 2);
    }
}

/// Indicates how a frame was received.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameInfo {
    /// Whether the frame was received with low output power
    pub low_power: bool,
    /// How the frame was addressed
    pub frame_addressing: FrameAddressing,
    /// Whether the frame is an explorer frame
    pub explorer_frame: bool,
    /// Whether the frame is for a different node (promiscuous mode only)
    pub foreign_target_node: bool,
    // Whether the frame is from a different home ID
    pub foreign_home_id: bool,
}

impl Parsable for FrameInfo {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        let (
            foreign_home_id,
            foreign_target_node,
            explorer_frame,
            frame_addressing,
            _reserved_2,
            low_power,
            _reserved_0,
        ) = bits::bits((
            bool,
            bool,
            bool,
            FrameAddressing::parse,
            u1::parse,
            bool,
            u1::parse,
        ))
        .parse(i)?;

        Ok(Self {
            low_power,
            frame_addressing,
            explorer_frame,
            foreign_target_node,
            foreign_home_id,
        })
    }
}
