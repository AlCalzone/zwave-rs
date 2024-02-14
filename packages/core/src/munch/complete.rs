use super::{combinators, streaming, ParseError, ParseResult, Parser};
use bytes::BytesMut;

fn map_incomplete<O>(res: ParseResult<O>) -> ParseResult<O> {
    match res {
        Err(ParseError::Incomplete(_)) => Err(ParseError::recoverable(())),
        _ => res,
    }
}

pub fn take(count: impl Into<usize>) -> impl Parser<BytesMut> {
    combinators::map_res(streaming::take(count), map_incomplete)
}

pub fn take_while1(predicate: impl Fn(u8) -> bool) -> impl Parser<BytesMut> {
    combinators::map_res(streaming::take_while1(predicate), map_incomplete)
}

pub fn literal(lit: u8) -> impl Parser<u8> {
    combinators::map_res(streaming::literal(lit), map_incomplete)
}
