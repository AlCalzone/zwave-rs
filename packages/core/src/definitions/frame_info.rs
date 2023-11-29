use crate::encoding::{self, BitParsable, BitSerializable, Parsable, Serializable, WriteLastNBits};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{
    bits, bits::complete::take as take_bits, combinator::map, complete::bool, error::context,
    multi::count, number::complete::be_u16, number::complete::be_u8, sequence::tuple,
};
use ux::{u1, u2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum FrameAddressing {
    Singlecast = 0b00,
    Broadcast = 0b01,
    Multicast = 0b10,
}

impl BitParsable for FrameAddressing {
    fn parse(i: encoding::BitInput) -> encoding::BitParseResult<Self> {
        context(
            "FrameType",
            map(take_bits(2usize), |x: u8| {
                FrameAddressing::try_from(x).unwrap()
            }),
        )(i)
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
    fn parse(i: encoding::Input) -> crate::prelude::ParseResult<Self> {
        let (
            i,
            (
                foreign_home_id,
                foreign_target_node,
                explorer_frame,
                frame_addressing,
                _reserved_2,
                low_power,
                _reserved_0,
            ),
        ) = bits(tuple((
            bool,
            bool,
            bool,
            FrameAddressing::parse,
            u1::parse,
            bool,
            u1::parse,
        )))(i)?;

        Ok((
            i,
            Self {
                low_power,
                frame_addressing,
                explorer_frame,
                foreign_target_node,
                foreign_home_id,
            },
        ))
    }
}
