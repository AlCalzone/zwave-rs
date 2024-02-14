use super::{combinators, Needed, ParseError, Parser};
use bytes::BytesMut;

pub fn take(count: impl Into<usize>) -> impl Parser<BytesMut> {
    let count: usize = count.into();
    move |input: &mut BytesMut| {
        let needed = count.saturating_sub(input.len());
        if needed > 0 {
            Err(ParseError::Incomplete(Needed::Size(needed)))
        } else {
            let output = input.split_to(count);
            Ok(output)
        }
    }
}

pub fn take_while0(predicate: impl Fn(u8) -> bool) -> impl Parser<BytesMut> {
    move |input: &mut BytesMut| {
        let end_pos = input.iter().position(|v| !predicate(*v));
        let ret = match end_pos {
            Some(pos) => input.split_to(pos),
            None => input.split(),
        };
        Ok(ret)
    }
}

pub fn take_while1(predicate: impl Fn(u8) -> bool) -> impl Parser<BytesMut> {
    move |input: &mut BytesMut| {
        if input.is_empty() {
            return Err(ParseError::Incomplete(Needed::Size(1)));
        }

        let end_pos = input.iter().position(|v| !predicate(*v));
        let ret = match end_pos {
            // We need at least one byte that matches the predicate
            Some(0) => return Err(ParseError::Incomplete(Needed::Size(1))),
            Some(pos) => input.split_to(pos),
            None => input.split(),
        };
        Ok(ret)
    }
}

pub fn literal(lit: u8) -> impl Parser<u8> {
    combinators::map_res(take(1usize), move |b| match b {
        Ok(b) if b[0] == lit => Ok(lit),
        Ok(_) => Err(ParseError::recoverable(())),
        Err(e) => Err(e),
    })
}
