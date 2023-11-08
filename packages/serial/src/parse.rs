// Heavily inspired from https://fasterthanli.me/series/making-our-own-ping/

use std::fmt;
use std::ops::RangeFrom;

use cookie_factory::GenError;
use nom::error::{
    ContextError as NomContextError, ErrorKind as NomErrorKind, ParseError as NomParseError,
};
use nom::{ErrorConvert, Slice};

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Nom(NomErrorKind),
    Context(&'static str),
    Validation(String),
}

#[derive(PartialEq)]
pub struct Error<I> {
    pub errors: Vec<(I, ErrorKind)>,
}

impl<I> Error<I> {
    fn validation_failure(input: I, reason: String) -> Self {
        let errors = vec![(input, ErrorKind::Validation(reason))];
        Self { errors }
    }
}

/// Validates that the given condition is satisfied, otherwise results in a
/// nom Failure with the given error message.
pub fn validate(input: Input, condition: bool, message: String) -> Result<()> {
    match condition {
        true => Ok((input, ())),
        false => Err(nom::Err::Failure(Error::validation_failure(input, message))),
    }
}

impl<I> NomParseError<I> for Error<I> {
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

impl<I> NomContextError<I> for Error<I> {
    // new!
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Context(ctx)));
        other
    }
}

impl<I> ErrorConvert<Error<I>> for Error<(I, usize)>
where
    I: Slice<RangeFrom<usize>>,
{
    fn convert(self) -> Error<I> {
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
        Error { errors }
    }
}

impl<'a> fmt::Debug for Error<&'a [u8]> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parsing error\n")?;

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
                write!(f, "\n")?;

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
                write!(f, "\n")?;

                Ok(())
            };

        for (input, kind) in self.errors.iter().rev() {
            let prefix = match kind {
                ErrorKind::Context(ctx) => format!("...in {}", ctx),
                ErrorKind::Nom(err) => format!("nom error {:?}", err),
                ErrorKind::Validation(reason) => format!("validation error: {}", reason),
            };

            write!(f, "{}\n", prefix)?;
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

pub type Input<'a> = &'a [u8];
pub type Result<'a, T> = nom::IResult<Input<'a>, T, Error<Input<'a>>>;

pub type BitInput<'a> = (&'a [u8], usize);
pub type BitResult<'a, T> = nom::IResult<BitInput<'a>, T, Error<BitInput<'a>>>;

pub trait BitParsable
where
    Self: Sized,
{
    fn parse(i: BitInput) -> BitResult<Self>;
}

// Convert all errors into a ParseError, while preserving validation errors
impl<T> crate::error::IntoResult for Result<'_, T> {
    type Output = T;

    fn into_result(self) -> crate::error::Result<Self::Output> {
        use crate::error::Error;
        let reason = match self {
            Ok((_, output)) => return Ok(output),

            Err(nom::Err::Incomplete(_)) | Err(nom::Err::Error(_)) => None,
            Err(nom::Err::Failure(e)) => {
                // Try to extract the failure reason
                e.errors.iter().find_map(|(_, kind)| match kind {
                    ErrorKind::Validation(reason) => Some(reason.clone()),
                    _ => None,
                })
            }
        };
        Err(Error::Parser(reason))
    }
}

impl<T> crate::error::IntoResult for BitResult<'_, T> {
    type Output = T;

    fn into_result(self) -> crate::error::Result<Self::Output> {
        use crate::error::Error;
        let reason = match self {
            Ok((_, output)) => return Ok(output),

            Err(nom::Err::Incomplete(_)) | Err(nom::Err::Error(_)) => None,
            Err(nom::Err::Failure(e)) => {
                // Try to extract the failure reason
                e.errors.iter().find_map(|(_, kind)| match kind {
                    ErrorKind::Validation(reason) => Some(reason.clone()),
                    _ => None,
                })
            }
        };
        Err(Error::Parser(reason))
    }
}

impl<T> crate::error::IntoResult for std::result::Result<T, GenError> {
    type Output = T;
    fn into_result(self) -> crate::error::Result<Self::Output> {
        use crate::error::Error;
        self.map_err(|e| Error::Serialization(format!("{:?}", e)))
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
                    fn parse(i: BitInput) -> BitResult<Self> {
                        map(take_bits($width as usize), Self::new)(i)
                    }
                }
            }
        )*
    };
}

impl_bit_parsable_for_ux!(1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15);

macro_rules! impl_vec_conversion_for_serializable {
    ($struct_name:ident) => {
        impl TryFrom<&[u8]> for $struct_name {
            type Error = crate::error::Error;
        
            fn try_from(value: &[u8]) -> crate::error::Result<Self> {
                use crate::error::IntoResult;
                Self::parse(value).into_result()
            }
        }
        
        impl TryInto<Vec<u8>> for &$struct_name {
            type Error = crate::error::Error;
        
            fn try_into(self) -> Result<Vec<u8>, Self::Error> {
                use crate::error::IntoResult;
                cf::gen_simple(self.serialize(), Vec::new()).into_result()
            }
        }
        
        impl TryInto<Vec<u8>> for $struct_name {
            type Error = crate::error::Error;
        
            fn try_into(self) -> Result<Vec<u8>, Self::Error> {
                (&self).try_into()
            }
        }
    };
}
