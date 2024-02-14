use super::{combinators, streaming, ParseError, ParseResult, Parser};
use bytes::Bytes;

fn map_incomplete<O>(res: ParseResult<O>) -> ParseResult<O> {
    match res {
        Err(ParseError::Incomplete(_)) => Err(ParseError::recoverable(())),
        _ => res,
    }
}

pub fn take(count: impl Into<usize>) -> impl Parser<Bytes> {
    let parser = streaming::take(count);
    move |input: &mut Bytes| {
        let res = parser.parse(input);
        map_incomplete(res)
    }
}

pub fn take_while1(predicate: impl Fn(u8) -> bool) -> impl Parser<Bytes> {
    let parser = streaming::take_while1(predicate);
    move |input: &mut Bytes| {
        let res: Result<Bytes, ParseError> = parser.parse(input);
        map_incomplete(res)
    }
}

pub fn literal(lit: u8) -> impl Parser<u8> {
    let parser = streaming::literal(lit);
    move |input: &mut Bytes| {
        let res = parser.parse(input);
        map_incomplete(res)
    }
}

// Consumes the given number of bytes without producing any output
pub fn skip(count: impl Into<usize>) -> impl Parser<()> {
    combinators::map(take(count), |_| ())
}
