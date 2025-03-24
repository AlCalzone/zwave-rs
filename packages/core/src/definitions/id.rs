use std::fmt::{Debug, Display, LowerHex, UpperHex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id16(u16);

impl Debug for Id16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}

impl Display for Id16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<u16> for Id16 {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Id16> for u16 {
    fn from(value: Id16) -> Self {
        value.0
    }
}

impl LowerHex for Id16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl UpperHex for Id16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Id32(u32);

impl Debug for Id32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

impl Display for Id32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<u32> for Id32 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Id32> for u32 {
    fn from(value: Id32) -> Self {
        value.0
    }
}

impl LowerHex for Id32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl UpperHex for Id32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}
