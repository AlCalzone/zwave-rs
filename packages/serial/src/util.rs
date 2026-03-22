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

/// Round a finite f32 to the nearest integer (ties away from zero).
/// Only valid for finite values within `i32` range — intended for small
/// values like dBm power levels. On `no_std` targets `f32::round()` is
/// unavailable, so we use an integer cast instead.
pub fn round_f32(x: f32) -> f32 {
    if x >= 0.0 {
        (x + 0.5) as i32 as f32
    } else {
        (x - 0.5) as i32 as f32
    }
}
