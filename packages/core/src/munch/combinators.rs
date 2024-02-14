use super::{Alt, ErrorContext, ParseError, Parser};
use bytes::Bytes;

pub fn map<O1, O2, P, F>(parser: P, f: F) -> impl Parser<O2>
where
    P: Parser<O1>,
    F: Fn(O1) -> O2,
{
    move |input: &mut bytes::Bytes| {
        let o1 = parser.parse(input)?;
        Ok(f(o1))
    }
}

pub fn map_res<O1, O2, P, F, E>(parser: P, f: F) -> impl Parser<O2>
where
    P: Parser<O1>,
    F: Fn(O1) -> Result<O2, E>,
    E: Into<ParseError>,
{
    move |input: &mut bytes::Bytes| {
        let o1 = parser.parse(input)?;
        f(o1).map_err(|e| e.into())
    }
}

pub fn peek<O, P>(parser: P) -> impl Parser<O>
where
    P: Parser<O>,
{
    // To peek a Bytes, simply clone it and parse the clone
    move |input: &mut Bytes| {
        let mut input_clone = input.clone();
        parser.parse(&mut input_clone)
    }
}

pub fn value<O, P>(parser: P, value: O) -> impl Parser<O>
where
    O: Copy,
    P: Parser<Bytes>,
{
    map(parser, move |_| value)
}

pub fn alt<O, List>(parsers: List) -> impl Parser<O>
where
    List: Alt<O>,
{
    move |input: &mut Bytes| parsers.choice(input)
}

/// Provides context to parse errors
pub fn context<O, P, C>(ctx: C, parser: P) -> impl Parser<O>
where
    P: Parser<O>,
    C: Clone + Into<ErrorContext>,
{
    move |input: &mut Bytes| {
        let res = parser.parse(input);
        match res {
            Err(ParseError::Recoverable(_)) => Err(ParseError::Recoverable(ctx.clone().into())),
            Err(ParseError::Final(_)) => Err(ParseError::Final(ctx.clone().into())),
            Err(ParseError::Incomplete(n)) => Err(ParseError::Incomplete(n)),
            Ok(o) => Ok(o),
        }
    }
}
