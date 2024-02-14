use super::{Needed, ParseError, Parser};
use bytes::{Buf, Bytes};

pub fn take(count: impl Into<usize>) -> impl Parser<Bytes> {
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

pub fn take_while0(predicate: impl Fn(u8) -> bool) -> impl Parser<Bytes> {
    move |input: &mut Bytes| {
        let end_pos = input.iter().position(|v| !predicate(*v));
        let ret = match end_pos {
            Some(pos) => input.split_to(pos),
            None => input.split_to(input.len()),
        };
        Ok(ret)
    }
}

pub fn take_while1(predicate: impl Fn(u8) -> bool) -> impl Parser<Bytes> {
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

pub fn literal(lit: u8) -> impl Parser<u8> {
    move |input: &mut Bytes| {
        let b = take(1usize).parse(input)?.get_u8();
        if b == lit {
            Ok(lit)
        } else {
            Err(ParseError::recoverable(()))
        }
    }
}
