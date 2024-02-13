use bytes::BytesMut;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Needed {
    Unknown,
    Size(usize),
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError<E> {
    #[error("Incomplete data: {0:?} bytes needed")]
    Incomplete(Needed),
    #[error("Recoverable error: {0}")]
    Recoverable(E),
    #[error("Final error: {0}")]
    Final(E),
}

pub type ParseResult<O, E = ()> = Result<O, ParseError<E>>;

pub trait Parser<O, E = ()> {
    /// Execute the parser on the input, advancing the input
    fn parse(&self, input: &mut BytesMut) -> ParseResult<O, E>;

    /// Execute the parser on the input, advancing the input only in case of success
    fn parse_peek(&self, input: &mut BytesMut) -> ParseResult<O, E> {
        let checkpoint = input.clone();
        let res = self.parse(input);
        if res.is_err() {
            *input = checkpoint;
        }
        res
    }
}

impl<O, E, F> Parser<O, E> for F
where
    F: Fn(&mut BytesMut) -> ParseResult<O, E>,
{
    fn parse(&self, input: &mut BytesMut) -> ParseResult<O, E> {
        self(input)
    }
}

pub mod streaming {
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
            _ => Err(ParseError::Recoverable(())),
        })
    }
}

pub mod complete {
    use super::{combinators, streaming, Needed, ParseError, ParseResult, Parser};
    use bytes::BytesMut;

    fn map_incomplete<O>(res: ParseResult<O>) -> ParseResult<O> {
        match res {
            Err(ParseError::Incomplete(_)) => Err(ParseError::Recoverable(())),
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
}

pub mod combinators {
    use super::{Needed, ParseError, ParseResult, Parser};
    use bytes::BytesMut;

    pub fn map<O1, O2, E, P, F>(parser: P, f: F) -> impl Parser<O2, E>
    where
        P: Parser<O1, E>,
        F: Fn(O1) -> O2,
    {
        move |input: &mut bytes::BytesMut| {
            let o1 = parser.parse(input)?;
            Ok(f(o1))
        }
    }

    pub fn map_res<O1, O2, E, P, F>(parser: P, f: F) -> impl Parser<O2, E>
    where
        P: Parser<O1, E>,
        F: Fn(ParseResult<O1, E>) -> ParseResult<O2, E>,
    {
        move |input: &mut bytes::BytesMut| {
            let res1 = parser.parse(input);
            f(res1)
        }
    }

    pub fn peek<O, E, P>(parser: P) -> impl Parser<O, E>
    where
        P: Parser<O, E>,
    {
        // To peek a BytesMut, simply clone it and parse the clone
        move |input: &mut BytesMut| {
            let mut input_clone = input.clone();
            parser.parse(&mut input_clone)
        }
    }

    pub fn value<O, E, P>(parser: P, value: O) -> impl Parser<O, E>
    where
        O: Copy,
        P: Parser<BytesMut, E>,
    {
        map(parser, move |_| value)
    }
}

pub mod multi {
    use super::{Needed, ParseError, ParseResult, Parser};
    use bytes::BytesMut;

    pub fn many_0<O, P>(parser: P) -> impl Parser<Vec<O>>
    where
        P: Parser<O>,
    {
        move |input: &mut BytesMut| {
            let mut output = Vec::new();
            while let Ok(o) = parser.parse_peek(input) {
                output.push(o);
            }
            Ok(output)
        }
    }

    pub fn many_n<O, P>(at_least: usize, parser: P) -> impl Parser<Vec<O>>
    where
        P: Parser<O>,
    {
        move |input: &mut BytesMut| {
            let checkpoint = input.clone();
            let mut output = Vec::new();
            while let Ok(o) = parser.parse_peek(input) {
                output.push(o);
            }
            if output.len() < at_least {
                *input = checkpoint;
                return Err(ParseError::Recoverable(()));
            }
            Ok(output)
        }
    }
}
