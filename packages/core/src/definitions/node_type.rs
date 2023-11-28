use crate::encoding::{self, BitParsable, BitSerializable, Parsable, Serializable, WriteLastNBits};

use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{bits::complete::take as take_bits, combinator::map};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum NodeType {
    Controller = 0,
    #[debug(format = "End Node")]
    EndNode = 1,
}

impl BitParsable for NodeType {
    fn parse(i: encoding::BitInput) -> encoding::BitParseResult<Self> {
        map(take_bits(1usize), |x: u8| NodeType::try_from(x).unwrap())(i)
    }
}

impl BitSerializable for NodeType {
    fn write(&self, b: &mut encoding::BitOutput) {
        b.write_last_n_bits((*self) as u8, 1);
    }
}
