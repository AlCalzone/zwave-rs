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
            Ok(_) => Err(ParseError::Recoverable(())),
            Err(e) => Err(e),
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
    use super::{Alt, Needed, ParseError, ParseResult, Parser};
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

    pub fn alt<O, E, List>(parsers: List) -> impl Parser<O, E>
    where
        List: Alt<O, E>,
    {
        move |input: &mut BytesMut| parsers.choice(input)
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

pub trait Alt<O, E = ()> {
    fn choice(&self, input: &mut BytesMut) -> ParseResult<O, E>;
}

macro_rules! impl_alt_trait {
    ($($idx:literal),+; $last:tt) => {
        paste::paste! {
            impl<$([<P $idx>]),+, [<P $last>], O, E> Alt<O, E> for ($([<P $idx>]),+, [<P $last>])
            where
            $(
                [<P $idx>]: Parser<O, E>,
            )+
                [<P $last>]: Parser<O, E>,
            {
                fn choice(&self, input: &mut BytesMut) -> ParseResult<O, E> {
                    $(
                        if let Ok(res) = self.$idx.parse_peek(input) {
                            return Ok(res);
                        }
                    )+
                    self.$last.parse(input)
                }
            }
        }
    };
    ($zero:literal) => {
        paste::paste! {
            impl<[<P $zero>], O, E> Alt<O, E> for ([<P $zero>],)
            where
                [<P $zero>]: Parser<O, E>,
            {
                fn choice(&self, input: &mut BytesMut) -> ParseResult<O, E> {
                    self.$zero.parse(input)
                }
            }
        }
    };
}

// TODO: There should be a more elegant solution for this
impl_alt_trait!(0);
impl_alt_trait!(0; 1);
impl_alt_trait!(0, 1; 2);
impl_alt_trait!(0, 1, 2; 3);
impl_alt_trait!(0, 1, 2, 3; 4);
impl_alt_trait!(0, 1, 2, 3, 4; 5);
impl_alt_trait!(0, 1, 2, 3, 4, 5; 6);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6; 7);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7; 8);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8; 9);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9; 10);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10; 11);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11; 12);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12; 13);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13; 14);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14; 15);
impl_alt_trait!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15; 16);

macro_rules! impl_parser_for_tuple {
    ($($idx:literal),+) => {
        paste::paste! {
            impl<$([<P $idx>], [<O $idx>]),+, E> Parser<($([<O $idx>]),+,), E> for ($([<P $idx>]),+,)
            where
            $(
                [<P $idx>]: Parser<[<O $idx>], E>,
            )+
            {
                fn parse(&self, input: &mut BytesMut) -> ParseResult<($([<O $idx>]),+,), E> {
                    Ok((
                        $(
                            self.$idx.parse(input)?,
                        )+
                    ))
                }
            }
        }
    };
}

impl_parser_for_tuple!(0);
impl_parser_for_tuple!(0, 1);
impl_parser_for_tuple!(0, 1, 2);
impl_parser_for_tuple!(0, 1, 2, 3);
impl_parser_for_tuple!(0, 1, 2, 3, 4);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
impl_parser_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

// impl<P0, O0, E> Parser<(O0,), E> for (P0,)
// where
//     P0: Parser<O0, E>,
// {
//     fn parse(&self, input: &mut BytesMut) -> ParseResult<(O0,), E> {
//         let o0 = self.0.parse(input)?;
//         Ok((o0,))
//     }
// }

// impl<P0, O0, P1, O1, E> Parser<(O0, O1), E> for (P0, P1)
// where
//     P0: Parser<O0, E>,
//     P1: Parser<O1, E>,
// {
//     fn parse(&self, input: &mut BytesMut) -> ParseResult<(O0, O1), E> {
//         let o0 = self.0.parse(input)?;
//         let o1 = self.1.parse(input)?;
//         Ok((o0, o1))
//     }
// }
