use ::core::fmt::Debug;
use core::fmt::{Formatter, Result};

pub fn hex_fmt<T: AsRef<[u8]>>(n: &T, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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

pub fn with_hex_fmt<T: core::fmt::Debug + AsRef<[u8]>>(n: &T) -> HexFmt<'_, T> {
    HexFmt { data: n }
}

/// Round an f32 to the nearest integer. Equivalent to f32::round() but
/// available on no_std targets where the inherent method requires std.
pub fn round_f32(x: f32) -> f32 {
    if x >= 0.0 {
        (x + 0.5) as i32 as f32
    } else {
        (x - 0.5) as i32 as f32
    }
}
