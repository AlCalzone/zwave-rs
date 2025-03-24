use ::core::fmt::Debug;
use core::fmt::{Formatter, Result};

pub fn hex_fmt<T: AsRef<[u8]>>(n: &T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "0x{}", hex::encode(n))
}

pub struct HexFmt<'a, T: 'a> {
    data: &'a T,
}
impl<'a, T: 'a + AsRef<[u8]>> Debug for HexFmt<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        hex_fmt(self.data, f)
    }
}

pub fn with_hex_fmt<T: std::fmt::Debug + AsRef<[u8]>>(n: &T) -> HexFmt<T> {
    HexFmt { data: n }
}
