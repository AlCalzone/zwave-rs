use crate::encoding::{self, BitParsable, BitSerializable, Parsable, Serializable, WriteLastNBits};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{
    bits::complete::take as take_bits,
    combinator::map,
    error::context,
    number::complete::{be_u16, be_u8},
};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
