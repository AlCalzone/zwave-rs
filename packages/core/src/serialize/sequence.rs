use super::{
    bytes::{be_u8, slice},
    Serializable,
};
use bitvec::prelude::*;
use bytes::BytesMut;

pub trait List {
    fn write_all(&self, output: &mut BytesMut);
}

pub fn tuple<L>(tuple: L) -> impl Serializable
where
    L: List,
{
    move |output: &mut BytesMut| tuple.write_all(output)
}

macro_rules! impl_list_for_tuple {
    ($($idx:literal),+) => {
        paste::paste! {
            impl<$([<E $idx>]),+> List for ($([<E $idx>]),+,)
            where
            $(
                [<E $idx>]: Serializable,
            )+
            {
                fn write_all(&self, output: &mut BytesMut) {
                    $(
                        self.$idx.serialize(output);
                    )+
                }
            }
        }
    };
}

impl_list_for_tuple!(0);
impl_list_for_tuple!(0, 1);
impl_list_for_tuple!(0, 1, 2);
impl_list_for_tuple!(0, 1, 2, 3);
impl_list_for_tuple!(0, 1, 2, 3, 4);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

/// Encodes a `Vec<u8>` as bitmask_length + bitmask where the least significant bit is mapped to `bit0_value`.
pub fn bitmask_u8<S: AsRef<[u8]>>(values: S, bit0_value: u8) -> impl Serializable {
    move |output: &mut BytesMut| {
        let values = values.as_ref();
        match values.len() {
            0 => be_u8(0u8).serialize(output),
            _ => {
                let indizes = values
                    .iter()
                    .map(|v| (v - bit0_value) as usize)
                    .collect::<Vec<_>>();

                let bit_len = indizes.iter().max().unwrap_or(&0) + 1;

                let mut bitvec = BitVec::<_, Lsb0>::new();
                bitvec.resize_with(bit_len, |idx| indizes.contains(&idx));
                let raw = bitvec.as_raw_slice();

                tuple((be_u8(raw.len() as u8), slice(raw))).serialize(output);
            }
        }
    }
}
