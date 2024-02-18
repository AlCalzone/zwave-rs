use super::{Needed, ParseError, ParseResult, Parser};
use bytes::{Buf, Bytes};

pub mod streaming {
    use super::*;

    pub fn take(count: impl Into<usize>) -> impl Parser<Bytes, Bytes> {
        let count: usize = count.into();
        move |input: &mut Bytes| {
            let needed = count.saturating_sub(input.len());
            if needed > 0 {
                Err(ParseError::Incomplete(Needed::Size(needed)))
            } else {
                let output = input.split_to(count);
                Ok(output)
            }
        }
    }

    pub fn take_while0(predicate: impl Fn(u8) -> bool) -> impl Parser<Bytes, Bytes> {
        move |input: &mut Bytes| {
            let end_pos = input.iter().position(|v| !predicate(*v));
            let ret = match end_pos {
                Some(pos) => input.split_to(pos),
                None => input.split_to(input.len()),
            };
            Ok(ret)
        }
    }

    pub fn take_while1(predicate: impl Fn(u8) -> bool) -> impl Parser<Bytes, Bytes> {
        move |input: &mut Bytes| {
            if input.is_empty() {
                return Err(ParseError::Incomplete(Needed::Size(1)));
            }

            let end_pos = input.iter().position(|v| !predicate(*v));
            let ret = match end_pos {
                // We need at least one byte that matches the predicate
                Some(0) => return Err(ParseError::Incomplete(Needed::Size(1))),
                Some(pos) => input.split_to(pos),
                None => input.split_to(input.len()),
            };
            Ok(ret)
        }
    }

    pub fn literal(lit: u8) -> impl Parser<Bytes, u8> {
        move |input: &mut Bytes| {
            let b = take(1usize).parse(input)?.get_u8();
            if b == lit {
                Ok(lit)
            } else {
                Err(ParseError::recoverable(()))
            }
        }
    }
}

pub mod complete {
    use super::*;
    use crate::parse::combinators;

    fn map_incomplete<O>(res: ParseResult<O>) -> ParseResult<O> {
        match res {
            Err(ParseError::Incomplete(_)) => Err(ParseError::recoverable(())),
            _ => res,
        }
    }

    pub fn take(count: impl Into<usize>) -> impl Parser<Bytes, Bytes> {
        let parser = streaming::take(count);
        move |input: &mut Bytes| {
            let res = parser.parse(input);
            map_incomplete(res)
        }
    }

    pub fn take_while1(predicate: impl Fn(u8) -> bool) -> impl Parser<Bytes, Bytes> {
        let parser = streaming::take_while1(predicate);
        move |input: &mut Bytes| {
            let res: Result<Bytes, ParseError> = parser.parse(input);
            map_incomplete(res)
        }
    }

    pub fn literal(lit: u8) -> impl Parser<Bytes, u8> {
        let parser = streaming::literal(lit);
        move |input: &mut Bytes| {
            let res = parser.parse(input);
            map_incomplete(res)
        }
    }

    // Consumes the given number of bytes without producing any output
    pub fn skip(count: impl Into<usize>) -> impl Parser<Bytes, ()> {
        combinators::map(take(count), |_| ())
    }
}

pub fn rest(input: &mut Bytes) -> ParseResult<Bytes> {
    Ok(input.split_to(input.len()))
}

macro_rules! impl_int {
    ($un:ident, 1) => {
        paste::paste! {
            pub fn [<be_ $un>](input: &mut Bytes) -> ParseResult<$un> {
                if input.remaining() < 1 {
                    Err(ParseError::Incomplete(Needed::Size(1)))
                } else {
                    Ok(input.[<get_ $un>]())
                }
            }
        }
    };
    ($un:ident, $bytes:literal) => {
        paste::paste! {
            pub fn [<be_ $un>](input: &mut Bytes) -> ParseResult<$un> {
                if input.remaining() < $bytes {
                    Err(ParseError::Incomplete(Needed::Size($bytes)))
                } else {
                    Ok(input.[<get_ $un>]())
                }
            }

            pub fn [<le_ $un>](input: &mut Bytes) -> ParseResult<$un> {
                if input.remaining() < $bytes {
                    Err(ParseError::Incomplete(Needed::Size($bytes)))
                } else {
                    Ok(input.[<get_ $un _le>]())
                }
            }
        }
    };
}

impl_int!(u8, 1);
impl_int!(u16, 2);
impl_int!(u32, 4);
impl_int!(u64, 8);
impl_int!(i8, 1);
impl_int!(i16, 2);
impl_int!(i32, 4);
impl_int!(i64, 8);
