use super::ensure_capacity;
use super::BitOutput;
use crate::prelude::*;
use bitvec::prelude::*;
use bytes::BytesMut;

pub fn bits<F>(f: F) -> impl Serializable
where
    F: Fn(&mut BitOutput),
{
    move |output: &mut BytesMut| {
        let mut bo = BitOutput::new();
        f(&mut bo);

        let data = bo.as_raw_slice();
        ensure_capacity(output, data.len());
        output.extend_from_slice(data);
    }
}

trait WriteLastNBits {
    fn write_last_n_bits<B: BitStore>(&mut self, b: B, num_bits: usize);
}

impl WriteLastNBits for BitOutput {
    fn write_last_n_bits<B: BitStore>(&mut self, b: B, num_bits: usize) {
        let bitslice = b.view_bits::<Lsb0>();
        let start = bitslice.len() - num_bits;
        self.extend_from_bitslice(&bitslice[start..])
    }
}

macro_rules! impl_bit_serializable_for_ux {
    ($($width: expr),*) => {
        $(
            paste::item! {
                impl BitSerializable for ux::[<u $width>] {
                    fn write(&self, b: &mut BitOutput) {
                        b.write_last_n_bits(u16::from(*self), $width);
                    }
                }
            }
        )*
    };
}

impl_bit_serializable_for_ux!(1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15);

impl BitSerializable for bool {
    fn write(&self, b: &mut BitOutput) {
        b.push(*self);
    }
}
