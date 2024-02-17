// Heavily inspired from https://fasterthanli.me/series/making-our-own-ping/

use crate::munch::{self, Parser};
use bitvec::prelude::*;
use bytes::Bytes;
use custom_debug_derive::Debug;
use std::borrow::Cow;
use thiserror::Error;

/// Validates that the given condition is satisfied, otherwise results in a
/// Parse error with the given error message.
pub fn validate(condition: bool, message: impl Into<Cow<'static, str>>) -> munch::ParseResult<()> {
    match condition {
        true => Ok(()),
        false => Err(munch::ParseError::validation_failure(message)),
    }
}

/// Returns a Parse error indicating that this parser is not implemented yet.
pub fn parser_not_implemented<T>(message: impl Into<Cow<'static, str>>) -> munch::ParseResult<T> {
    Err(munch::ParseError::not_implemented(message))
}

#[derive(Error, Debug, PartialEq)]
pub enum TryFromReprError<T: std::fmt::Debug> {
    #[error("{0:?} is not a valid value for this enum")]
    Invalid(T),
    #[error("{0:?} resolves to a non-primitive enum variant")]
    NonPrimitive(T),
}

pub type BitOutput = BitVec<u8, Msb0>;

// FIXME: Get rid of this trait and use Parser instead
pub trait Parsable
where
    Self: Sized,
{
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self>;
}

pub trait BitParsable
where
    Self: Sized,
{
    fn parse(i: &mut (Bytes, usize)) -> crate::munch::ParseResult<Self>;
}

pub trait BitSerializable {
    fn write(&self, b: &mut BitOutput);
}

pub trait WriteLastNBits {
    fn write_last_n_bits<B: BitStore>(&mut self, b: B, num_bits: usize);
}

impl WriteLastNBits for BitOutput {
    fn write_last_n_bits<B: BitStore>(&mut self, b: B, num_bits: usize) {
        let bitslice = b.view_bits::<Lsb0>();
        let start = bitslice.len() - num_bits;
        self.extend_from_bitslice(&bitslice[start..])
    }
}

macro_rules! impl_bit_parsable_for_ux {
    ($($width: expr),*) => {
        $(
            paste::item! {
                impl BitParsable for ux::[<u $width>] {
                    fn parse(i: &mut (Bytes, usize)) -> munch::ParseResult<Self> {
                        use munch::{combinators::map, bits::take};
                        map(take($width as usize), Self::new).parse(i)
                    }
                }
            }
        )*
    };
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

impl_bit_parsable_for_ux!(1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15);
impl_bit_serializable_for_ux!(1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15);

impl BitSerializable for bool {
    fn write(&self, b: &mut BitOutput) {
        b.push(*self);
    }
}

pub mod encoders {
    use crate::bake::{
        bytes::{be_u8, slice},
        sequence::tuple,
        Encoder,
    };
    use bitvec::prelude::*;
    use bytes::BytesMut;

    /// Encodes a `Vec<u8>` as bitmask_length + bitmask where the least significant bit is mapped to `bit0_value`.
    pub fn bitmask_u8(values: &[u8], bit0_value: u8) -> impl Encoder + '_ {
        move |output: &mut BytesMut| match values.len() {
            0 => be_u8(0u8).write(output),
            _ => {
                let indizes = values
                    .iter()
                    .map(|v| (v - bit0_value) as usize)
                    .collect::<Vec<_>>();

                let bit_len = indizes.iter().max().unwrap_or(&0) + 1;

                let mut bitvec = BitVec::<_, Lsb0>::new();
                bitvec.resize_with(bit_len, |idx| indizes.contains(&idx));
                let raw = bitvec.as_raw_slice();

                tuple((be_u8(raw.len() as u8), slice(raw))).write(output);
            }
        }
    }
}

pub mod parsers {
    use super::Parsable;
    use crate::munch::{
        bytes::{
            be_u8,
            complete::{literal, take},
        },
        combinators::{map, map_parser},
        multi::{length_data, many_0, separated_pair},
    };
    use crate::prelude::*;
    use bitvec::prelude::*;
    use bytes::Bytes;

    /// Parses a bitmask into a `Vec<u8>`. The least significant bit is mapped to `bit0_value`. The first byte is considerd to be the bitmask length.
    pub fn variable_length_bitmask_u8(
        i: &mut Bytes,
        bit0_value: u8,
    ) -> crate::munch::ParseResult<Vec<u8>> {
        let bitmask = length_data(be_u8).parse(i)?;

        let view = bitmask.view_bits::<Lsb0>();
        let ret = view
            .iter_ones()
            .map(|index| (index as u8) + bit0_value)
            .collect::<Vec<_>>();
        Ok(ret)
    }

    /// Parses a bitmask with the given length into a `Vec<u8>`. The least significant bit is mapped to `bit0_value`.
    pub fn fixed_length_bitmask_u8(
        i: &mut Bytes,
        bit0_value: u8,
        bitmask_len: usize,
    ) -> crate::munch::ParseResult<Vec<u8>> {
        let bitmask = take(bitmask_len).parse(i)?;

        let view = bitmask.view_bits::<Lsb0>();
        let ret = view
            .iter_ones()
            .map(|index| (index as u8) + bit0_value)
            .collect::<Vec<_>>();
        Ok(ret)
    }

    /// Parses a list of supported and controlled CCs that starts with a length byte
    pub fn variable_length_cc_list(
        i: &mut Bytes,
    ) -> crate::munch::ParseResult<(
        Vec<CommandClasses>, // supported
        Vec<CommandClasses>, // controlled
    )> {
        map_parser(
            length_data(be_u8),
            separated_pair(
                many_0(CommandClasses::parse),
                literal(COMMAND_CLASS_SUPPORT_CONTROL_MARK),
                many_0(CommandClasses::parse),
            ),
        )
        .parse(i)
    }

    /// Parses a list of supported and controlled CCs with the given length
    pub fn fixed_length_cc_list(
        i: &mut Bytes,
        len: usize,
    ) -> crate::munch::ParseResult<(
        Vec<CommandClasses>, // supported
        Vec<CommandClasses>, // controlled
    )> {
        map_parser(
            take(len),
            separated_pair(
                many_0(CommandClasses::parse),
                literal(COMMAND_CLASS_SUPPORT_CONTROL_MARK),
                many_0(CommandClasses::parse),
            ),
        )
        .parse(i)
    }

    /// Parses a list of supported (NOT controlled) CCs with the given length
    pub fn fixed_length_cc_list_only_supported(
        i: &mut Bytes,
        len: usize,
    ) -> crate::munch::ParseResult<Vec<CommandClasses>> {
        map_parser(take(len), many_0(CommandClasses::parse)).parse(i)
    }

    pub fn version_major_minor_patch(i: &mut Bytes) -> crate::munch::ParseResult<Version> {
        map((be_u8, be_u8, be_u8), |(major, minor, patch)| Version {
            major,
            minor,
            patch: Some(patch),
        })
        .parse(i)
    }

    pub fn version_major_minor(i: &mut Bytes) -> crate::munch::ParseResult<Version> {
        map((be_u8, be_u8), |(major, minor)| Version {
            major,
            minor,
            patch: None,
        })
        .parse(i)
    }
}
