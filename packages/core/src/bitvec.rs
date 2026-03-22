use zwave_pal::prelude::*;

/// A growable bit vector that packs bits MSB-first into bytes.
///
/// Bits are written left-to-right within each byte (most significant bit first),
/// matching the Z-Wave serial protocol's bit ordering for serialization.
pub struct BitVec {
    bytes: Vec<u8>,
    /// Total number of bits written.
    bit_len: usize,
}

impl Default for BitVec {
    fn default() -> Self {
        Self::new()
    }
}

impl BitVec {
    /// Creates a new empty bit vec.
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_len: 0,
        }
    }

    /// Appends a single bit (MSB-first ordering within each byte).
    pub fn push(&mut self, bit: bool) {
        let byte_idx = self.bit_len / 8;
        let bit_idx = 7 - (self.bit_len % 8); // MSB-first
        if byte_idx >= self.bytes.len() {
            self.bytes.push(0);
        }
        if bit {
            self.bytes[byte_idx] |= 1 << bit_idx;
        }
        self.bit_len += 1;
    }

    /// Appends the lowest `count` bits of `value`, MSB-first.
    ///
    /// For example, `push_bits(0b101, 3)` pushes bits 1, 0, 1 in that order.
    pub fn push_bits(&mut self, value: u16, count: usize) {
        for i in (0..count).rev() {
            self.push((value >> i) & 1 != 0);
        }
    }

    /// Returns the underlying bytes. Trailing bits in the last byte are zero-padded.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Iterates over the indices of set bits in a byte slice (LSB-first ordering).
///
/// Bit 0 of byte 0 is index 0, bit 1 of byte 0 is index 1, ...,
/// bit 0 of byte 1 is index 8, etc.
pub fn iter_ones(bytes: &[u8]) -> impl Iterator<Item = usize> + '_ {
    bytes.iter().enumerate().flat_map(|(byte_idx, &byte)| {
        (0..8u8)
            .filter(move |&bit| byte & (1 << bit) != 0)
            .map(move |bit| byte_idx * 8 + bit as usize)
    })
}

/// Builds a bitmask byte array from a set of bit indices (LSB-first ordering).
///
/// Each index in `indices` sets the corresponding bit in the output.
pub fn build_bitmask(indices: &[usize], bit_len: usize) -> Vec<u8> {
    let byte_len = bit_len.div_ceil(8);
    let mut bitmask = vec![0u8; byte_len];
    for &idx in indices {
        if idx < bit_len {
            bitmask[idx / 8] |= 1 << (idx % 8); // LSB-first
        }
    }
    bitmask
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[expect(clippy::unusual_byte_groupings)]
    fn test_push_bits_msb() {
        let mut buf = BitVec::new();
        buf.push_bits(0b10110, 5);
        buf.push_bits(0b101, 3);
        assert_eq!(buf.as_bytes(), &[0b10110_101]);
    }

    #[test]
    fn test_push_bool() {
        let mut buf = BitVec::new();
        buf.push(true);
        buf.push(false);
        buf.push(true);
        buf.push(true);
        buf.push(false);
        buf.push(false);
        buf.push(false);
        buf.push(false);
        assert_eq!(buf.as_bytes(), &[0b10110000]);
    }

    #[test]
    fn test_push_across_byte_boundary() {
        let mut buf = BitVec::new();
        buf.push_bits(0xFF, 8);
        buf.push_bits(0b10, 2);
        assert_eq!(buf.as_bytes(), &[0xFF, 0b10_000000]);
    }

    #[test]
    fn test_iter_set_bits() {
        let bytes = [0b00000101, 0b00000010]; // bits 0,2 in byte0; bit 1 in byte1
        let bits: Vec<usize> = iter_ones(&bytes).collect();
        assert_eq!(bits, vec![0, 2, 9]);
    }

    #[test]
    fn test_build_bitmask() {
        let mask = build_bitmask(&[0, 2, 9], 16);
        assert_eq!(mask, vec![0b00000101, 0b00000010]);
    }

    #[test]
    fn test_roundtrip_bitmask() {
        let indices = vec![1, 3, 8, 11];
        let mask = build_bitmask(&indices, 16);
        let recovered: Vec<usize> = iter_ones(&mask).collect();
        assert_eq!(recovered, indices);
    }
}
