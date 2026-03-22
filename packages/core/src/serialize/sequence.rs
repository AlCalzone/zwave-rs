use super::{
    Serializable,
    bytes::{be_u8, slice},
};
use crate::bitvec::build_bitmask;
use bytes::BytesMut;
use zwave_pal::prelude::*;

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

fn encode_bitmask_values(values: &[u8], bit0_value: u8, bit_len: usize) -> Vec<u8> {
    let indices: Vec<usize> = values
        .iter()
        .filter_map(|value| {
            value
                .checked_sub(bit0_value)
                .map(|index| index as usize)
                .filter(|index| *index < bit_len)
        })
        .collect();

    build_bitmask(&indices, bit_len)
}

/// Encodes a `Vec<u8>` as bitmask_length + bitmask where the least significant bit is mapped to `bit0_value`.
pub fn bitmask_u8<S: AsRef<[u8]>>(values: S, bit0_value: u8) -> impl Serializable {
    move |output: &mut BytesMut| {
        let values = values.as_ref();
        let indices = values
            .iter()
            .filter_map(|value| value.checked_sub(bit0_value).map(|index| index as usize))
            .collect::<Vec<_>>();

        match indices.len() {
            0 => be_u8(0u8).serialize(output),
            _ => {
                let bit_len = indices.iter().max().unwrap_or(&0) + 1;
                let raw = encode_bitmask_values(values, bit0_value, bit_len);

                tuple((be_u8(raw.len() as u8), slice(raw))).serialize(output);
            }
        }
    }
}

/// Encodes a fixed-length bitmask where the least significant bit is mapped to `bit0_value`.
pub fn fixed_length_bitmask_u8<S: AsRef<[u8]>>(
    values: S,
    bit0_value: u8,
    bitmask_len: usize,
) -> impl Serializable {
    move |output: &mut BytesMut| {
        let raw = encode_bitmask_values(values.as_ref(), bit0_value, bitmask_len * 8);
        slice(raw).serialize(output);
    }
}

#[cfg(test)]
mod tests {
    use super::{bitmask_u8, fixed_length_bitmask_u8};
    use crate::serialize::Serializable;
    use bytes::BytesMut;

    #[test]
    fn variable_length_bitmask_u8_prefixes_the_encoded_length() {
        let mut output = BytesMut::new();
        bitmask_u8([1u8, 3u8], 1).serialize(&mut output);

        assert_eq!(output.as_ref(), &[1, 0b0000_0101]);
    }

    #[test]
    fn fixed_length_bitmask_u8_omits_the_length_prefix() {
        let mut output = BytesMut::new();
        fixed_length_bitmask_u8([0u8, 7u8], 0, 1).serialize(&mut output);

        assert_eq!(output.as_ref(), &[0b1000_0001]);
    }
}
