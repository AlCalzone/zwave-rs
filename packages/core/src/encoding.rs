// Heavily inspired from https://fasterthanli.me/series/making-our-own-ping/

use bitvec::prelude::*;
use cookie_factory::GenError;
use custom_debug_derive::Debug;
use nom::error::{
    ContextError as NomContextError, ErrorKind as NomErrorKind, FromExternalError,
    ParseError as NomParseError,
};
use nom::{ErrorConvert, Slice};
use std::fmt;
use std::ops::RangeFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Nom(NomErrorKind),
    Context(&'static str),
    Validation(String),
    NotImplemented(&'static str),
}

#[derive(PartialEq)]
pub struct NomError<I> {
    pub errors: Vec<(I, ErrorKind)>,
}

impl<I> NomError<I> {
    pub fn validation_failure(input: I, reason: String) -> Self {
        let errors = vec![(input, ErrorKind::Validation(reason))];
        Self { errors }
    }

    pub fn not_implemented(input: I, message: &'static str) -> Self {
        let errors = vec![(input, ErrorKind::NotImplemented(message))];
        Self { errors }
    }
}

/// Validates that the given condition is satisfied, otherwise results in a
/// nom Failure with the given error message.
pub fn validate(input: Input, condition: bool, message: String) -> ParseResult<()> {
    match condition {
        true => Ok((input, ())),
        false => Err(nom::Err::Failure(NomError::validation_failure(
            input, message,
        ))),
    }
}

/// Returns a nom Failure indicating that this parser is not implemented yet.
pub fn parser_not_implemented<'a, T>(
    input: Input<'a>,
    message: &'static str,
) -> ParseResult<'a, T> {
    Err(nom::Err::Failure(NomError::not_implemented(input, message)))
}

impl<I> NomParseError<I> for NomError<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        let errors = vec![(input, ErrorKind::Nom(kind))];
        Self { errors }
    }

    fn append(input: I, kind: NomErrorKind, mut other: Self) -> Self {
        // was (input, kind)
        other.errors.push((input, ErrorKind::Nom(kind)));
        other
    }
}

impl<I> NomContextError<I> for NomError<I> {
    // new!
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Context(ctx)));
        other
    }
}

impl<I> ErrorConvert<NomError<I>> for NomError<(I, usize)>
where
    I: Slice<RangeFrom<usize>>,
{
    fn convert(self) -> NomError<I> {
        // alright pay close attention.
        // `self` (the input) is a bit-level error. since it's
        // our custom error type, it can contain multiple errors,
        // each with its own location. so we need to convert them all
        // from bit-level to byte-level
        let errors = self
            .errors
            // this moves every element of `self.errors` into the
            // iterator, whereas `iter()` would have given us references.
            .into_iter()
            // this converts bit-level positions to byte-level positions
            // (ie. plain old slices). If we're not on a byte boundary,
            // we take the closest byte boundary to the left.
            .map(|((rest, offset), err)| (rest.slice(offset / 8..), err))
            // this gives us a Vec again
            .collect();
        NomError { errors }
    }
}

impl<'a> fmt::Debug for NomError<&'a [u8]> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "parsing error")?;

        let mut shown_input = None;
        let margin_left = 4;
        let margin_str = " ".repeat(margin_left);

        // maximum amount of binary data we'll dump per line
        let maxlen = 60;

        // given a big slice, an offset, and a length, attempt to show
        // some data before, some data after, and highlight which part
        // we're talking about with tildes.
        let print_slice =
            |f: &mut fmt::Formatter, s: &[u8], offset: usize, len: usize| -> fmt::Result {
                // decide which part of `s` we're going to show.
                let (s, offset, len) = {
                    // see diagram further in article.
                    // TODO: review for off-by-one errors

                    let avail_after = s.len() - offset;
                    let after = std::cmp::min(avail_after, maxlen / 2);

                    let avail_before = offset;
                    let before = std::cmp::min(avail_before, maxlen / 2);

                    let new_start = offset - before;
                    let new_end = offset + after;
                    let new_offset = before;
                    let new_len = std::cmp::min(new_end - new_start, len);

                    (&s[new_start..new_end], new_offset, new_len)
                };

                write!(f, "{}", margin_str)?;
                for b in s {
                    write!(f, "{:02X} ", b)?;
                }
                writeln!(f)?;

                write!(f, "{}", margin_str)?;
                for i in 0..s.len() {
                    // each byte takes three characters, ie "FF "
                    if i == offset + len - 1 {
                        // ..except the last one
                        write!(f, "~~")?;
                    } else if (offset..offset + len).contains(&i) {
                        write!(f, "~~~")?;
                    } else {
                        write!(f, "   ")?;
                    };
                }
                writeln!(f)?;

                Ok(())
            };

        for (input, kind) in self.errors.iter().rev() {
            let prefix = match kind {
                ErrorKind::Context(ctx) => format!("...in {}", ctx),
                ErrorKind::Nom(err) => format!("nom error {:?}", err),
                ErrorKind::Validation(reason) => format!("validation error: {}", reason),
                ErrorKind::NotImplemented(reason) => format!("not implemented: {}", reason),
            };

            writeln!(f, "{}", prefix)?;
            match shown_input {
                None => {
                    shown_input.replace(input);
                    print_slice(f, input, 0, input.len())?;
                }
                Some(parent_input) => {
                    // `nom::Offset` is a trait that lets us get the position
                    // of a subslice into its parent slice. This works great for
                    // our error reporting!
                    use nom::Offset;
                    let offset = parent_input.offset(input);
                    print_slice(f, parent_input, offset, input.len())?;
                }
            };
        }
        Ok(())
    }
}

pub struct NomTryFromPrimitiveError(String);

impl<I> FromExternalError<I, NomTryFromPrimitiveError> for NomError<I> {
    fn from_external_error(
        input: I,
        _kind: nom::error::ErrorKind,
        e: NomTryFromPrimitiveError,
    ) -> Self {
        NomError::validation_failure(input, e.0)
    }
}

/// A helper trait that allows using the `TryFrom` implementation generated by `derive_try_from_primitive`
/// with a nom parser. Adds the `try_from_primitive` method that returns a nom-compatible error if the
/// conversion fails.
pub trait NomTryFromPrimitive
where
    Self: Sized + TryFrom<Self::Repr>,
    <Self as NomTryFromPrimitive>::Repr: Copy,
{
    type Repr;

    fn format_error(repr: Self::Repr) -> String;

    fn try_from_primitive<T: Into<Self::Repr>>(repr: T) -> Result<Self, NomTryFromPrimitiveError> {
        let repr = repr.into();
        Self::try_from(repr).map_err(|_| NomTryFromPrimitiveError(Self::format_error(repr)))
    }
}

pub type Input<'a> = &'a [u8];
pub type ParseResult<'a, T> = nom::IResult<Input<'a>, T, NomError<Input<'a>>>;

pub type BitInput<'a> = (&'a [u8], usize);
pub type BitParseResult<'a, T> = nom::IResult<BitInput<'a>, T, NomError<BitInput<'a>>>;
pub type BitOutput = BitVec<u8, Msb0>;

pub trait Parsable
where
    Self: Sized,
{
    fn parse(i: Input) -> ParseResult<Self>;

    fn try_from_slice(data: &[u8]) -> Result<Self, EncodingError> {
        Self::parse(data).into_encoding_result()
    }
}

pub trait BitParsable
where
    Self: Sized,
{
    fn parse(i: BitInput) -> BitParseResult<Self>;
}

pub trait Serializable
where
    Self: Sized,
{
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a;

    fn try_to_vec(&self) -> Result<Vec<u8>, EncodingError> {
        cookie_factory::gen_simple(self.serialize(), Vec::new()).into_encoding_result()
    }
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

use nom::bits::streaming::take as take_bits;
use nom::combinator::map;

macro_rules! impl_bit_parsable_for_ux {
    ($($width: expr),*) => {
        $(
            paste::item! {
                use ux::[<u $width>];
                impl BitParsable for [<u $width>] {
                    fn parse(i: BitInput) -> BitParseResult<Self> {
                        map(take_bits($width as usize), Self::new)(i)
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
                impl BitSerializable for [<u $width>] {
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

impl<T> Serializable for Option<T>
where
    T: Serializable,
{
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |out| match self {
            Some(v) => v.serialize()(out),
            None => crate::encoding::encoders::empty()(out),
        }
    }
}

/// A simple result type concerning conversion from and to binary data
pub type EncodingResult<T> = std::result::Result<T, EncodingError>;

#[derive(Error, Debug)]
/// A simple error type concerning conversion from/to binary data
pub enum EncodingError {
    #[error("Parse error: {0:?}")]
    Parse(Option<String>),
    #[error("Serialization error: {0:?}")]
    Serialize(String),
    #[error("Not implemented: {0:?}")]
    NotImplemented(&'static str),
}

impl<I> FromExternalError<I, EncodingError> for NomError<I> {
    fn from_external_error(input: I, _kind: nom::error::ErrorKind, e: EncodingError) -> Self {
        NomError::validation_failure(input, e.to_string())
    }
}

/// A simple result type concerning conversion from and to binary data
pub type SimpleParseResult<T> = std::result::Result<T, SimpleParseError>;

#[derive(Error, Debug)]
#[error("Parse error: {0:?}")]
/// A simple error type concerning parsing things
pub struct SimpleParseError(pub Option<String>);

/// Provides a way to convert custom results into this library's result type
/// without breaking the orphan rule
pub trait IntoEncodingResult {
    type Output;
    fn into_encoding_result(self) -> EncodingResult<Self::Output>;
}

// Convert all errors into a ParseError, while preserving validation errors
impl<T> IntoEncodingResult for ParseResult<'_, T> {
    type Output = T;

    fn into_encoding_result(self) -> EncodingResult<Self::Output> {
        match self {
            Ok((_, output)) => Ok(output),

            Err(nom::Err::Incomplete(_)) | Err(nom::Err::Error(_)) => {
                Err(EncodingError::Parse(None))
            }
            Err(nom::Err::Failure(e)) => {
                // Try to extract the failure reason
                let err = e.errors.iter().find_map(|(_, kind)| match kind {
                    ErrorKind::NotImplemented(reason) => {
                        Some(EncodingError::NotImplemented(reason))
                    }
                    ErrorKind::Validation(reason) => {
                        Some(EncodingError::Parse(Some(reason.clone())))
                    }
                    _ => None,
                });
                Err(err.unwrap_or(EncodingError::Parse(None)))
            }
        }
    }
}

impl<T> IntoEncodingResult for BitParseResult<'_, T> {
    type Output = T;

    fn into_encoding_result(self) -> EncodingResult<Self::Output> {
        match self {
            Ok((_, output)) => Ok(output),

            Err(nom::Err::Incomplete(_)) | Err(nom::Err::Error(_)) => {
                Err(EncodingError::Parse(None))
            }
            Err(nom::Err::Failure(e)) => {
                // Try to extract the failure reason
                let err = e.errors.iter().find_map(|(_, kind)| match kind {
                    ErrorKind::NotImplemented(reason) => {
                        Some(EncodingError::NotImplemented(reason))
                    }
                    ErrorKind::Validation(reason) => {
                        Some(EncodingError::Parse(Some(reason.clone())))
                    }
                    _ => None,
                });
                Err(err.unwrap_or(EncodingError::Parse(None)))
            }
        }
    }
}

impl From<GenError> for EncodingError {
    fn from(e: GenError) -> Self {
        EncodingError::Serialize(format!("{:?}", e))
    }
}

impl<T> IntoEncodingResult for std::result::Result<T, GenError> {
    type Output = T;

    fn into_encoding_result(self) -> EncodingResult<Self::Output> {
        self.map_err(EncodingError::from)
    }
}

impl From<std::io::Error> for EncodingError {
    fn from(e: std::io::Error) -> Self {
        // IO Errors should only happen while writing to the serial port
        EncodingError::Serialize(format!("{:?}", e))
    }
}

pub mod encoders {
    use super::BitOutput;
    use bitvec::prelude::*;
    use cookie_factory as cf;
    use std::io;

    pub fn bits<W, F>(f: F) -> impl cf::SerializeFn<W>
    where
        W: io::Write,
        F: Fn(&mut BitOutput),
    {
        move |mut out: cf::WriteContext<W>| {
            let mut bo = BitOutput::new();
            f(&mut bo);

            io::Write::write(&mut out, bo.as_raw_slice())?;
            Ok(out)
        }
    }
    /// A SerializeFn that does nothing
    pub fn empty<W: std::io::Write>() -> impl cookie_factory::SerializeFn<W> {
        move |out: cookie_factory::WriteContext<W>| Ok(out)
    }

    /// Encodes a `Vec<u8>` as bitmask_length + bitmask where the least significant bit is mapped to `bit0_value`.
    pub fn bitmask_u8<'a, W: std::io::Write + 'a>(
        values: &'a [u8],
        bit0_value: u8,
    ) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |out| match values.len() {
            0 => cf::bytes::be_u8(0u8)(out),
            _ => {
                let indizes = values
                    .iter()
                    .map(|v| (v - bit0_value) as usize)
                    .collect::<Vec<_>>();

                let bit_len = indizes.iter().max().unwrap_or(&0) + 1;

                let mut bitvec = BitVec::<_, Lsb0>::new();
                bitvec.resize_with(bit_len, |idx| indizes.contains(&idx));
                let raw = bitvec.as_raw_slice().to_owned();

                cf::sequence::tuple((
                    cf::bytes::be_u8(raw.len() as u8),
                    cf::combinator::slice(raw),
                ))(out)
            }
        }
    }
}

pub mod parsers {
    use bitvec::prelude::*;
    use nom::bytes::complete::{tag, take as take_bytes};
    use nom::combinator::map_parser;
    use nom::multi::{length_data, many0};
    use nom::number::complete::be_u8;
    use nom::sequence::separated_pair;

    use crate::prelude::{CommandClasses, COMMAND_CLASS_SUPPORT_CONTROL_MARK};

    use super::Parsable;

    /// Parses a bitmask into a `Vec<u8>`. The least significant bit is mapped to `bit0_value`. The first byte is considerd to be the bitmask length.
    pub fn variable_length_bitmask_u8(
        i: super::Input,
        bit0_value: u8,
    ) -> super::ParseResult<Vec<u8>> {
        let (i, bitmask) = length_data(be_u8)(i)?;

        let view = bitmask.view_bits::<Lsb0>();
        let ret = view
            .iter_ones()
            .map(|index| (index as u8) + bit0_value)
            .collect::<Vec<_>>();
        Ok((i, ret))
    }

    /// Parses a bitmask with the given length into a `Vec<u8>`. The least significant bit is mapped to `bit0_value`.
    pub fn fixed_length_bitmask_u8(
        i: super::Input,
        bit0_value: u8,
        bitmask_len: usize,
    ) -> super::ParseResult<Vec<u8>> {
        let (i, bitmask) = take_bytes(bitmask_len)(i)?;

        let view = bitmask.view_bits::<Lsb0>();
        let ret = view
            .iter_ones()
            .map(|index| (index as u8) + bit0_value)
            .collect::<Vec<_>>();
        Ok((i, ret))
    }

    /// Parses a list of supported and controlled CCs that starts with a length byte
    pub fn variable_length_cc_list(
        i: super::Input,
    ) -> super::ParseResult<(
        Vec<CommandClasses>, // supported
        Vec<CommandClasses>, // controlled
    )> {
        map_parser(
            length_data(be_u8),
            separated_pair(
                many0(CommandClasses::parse),
                tag([COMMAND_CLASS_SUPPORT_CONTROL_MARK]),
                many0(CommandClasses::parse),
            ),
        )(i)
    }

    /// Parses a list of supported and controlled CCs with the given length
    pub fn fixed_length_cc_list(
        i: super::Input,
        len: usize,
    ) -> super::ParseResult<(
        Vec<CommandClasses>, // supported
        Vec<CommandClasses>, // controlled
    )> {
        map_parser(
            take_bytes(len),
            separated_pair(
                many0(CommandClasses::parse),
                tag([COMMAND_CLASS_SUPPORT_CONTROL_MARK]),
                many0(CommandClasses::parse),
            ),
        )(i)
    }

    /// Parses a list of supported (NOT controlled) CCs with the given length
    pub fn fixed_length_cc_list_only_supported(
        i: super::Input,
        len: usize,
    ) -> super::ParseResult<Vec<CommandClasses>> {
        map_parser(take_bytes(len), many0(CommandClasses::parse))(i)
    }
}
