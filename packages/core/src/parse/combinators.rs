use super::{Alt, ErrorContext, ParseError, Parser};

pub fn map<I, O1, O2, P, F>(parser: P, f: F) -> impl Parser<I, O2>
where
    I: Clone,
    P: Parser<I, O1>,
    F: Fn(O1) -> O2,
{
    move |input: &mut I| {
        let o1 = parser.parse(input)?;
        Ok(f(o1))
    }
}

pub fn map_res<I, O1, O2, P, F, E>(parser: P, f: F) -> impl Parser<I, O2>
where
    I: Clone,
    P: Parser<I, O1>,
    F: Fn(O1) -> Result<O2, E>,
    E: Into<ParseError>,
{
    move |input: &mut I| {
        let o1 = parser.parse(input)?;
        f(o1).map_err(|e| e.into())
    }
}

pub fn map_parser<I, O1, O2, P1, P2>(first: P1, second: P2) -> impl Parser<I, O2>
where
    I: Clone,
    O1: Clone,
    P1: Parser<I, O1>,
    P2: Parser<O1, O2>,
{
    move |input: &mut I| {
        let mut o1 = first.parse(input)?;
        second.parse(&mut o1)
    }
}

pub fn peek<I, O, P>(parser: P) -> impl Parser<I, O>
where
    I: Clone,
    P: Parser<I, O>,
{
    // To peek the input, simply clone it and parse the clone
    move |input: &mut I| {
        let mut input_clone = input.clone();
        parser.parse(&mut input_clone)
    }
}

pub fn value<I, V, P>(parser: P, value: V) -> impl Parser<I, V>
where
    I: Clone,
    V: Copy,
    P: Parser<I>,
{
    map(parser, move |_| value)
}

pub fn alt<I, O, List>(parsers: List) -> impl Parser<I, O>
where
    I: Clone,
    List: Alt<I, O>,
{
    move |input: &mut I| parsers.choice(input)
}

pub fn cond<I, O, P>(condition: bool, parser: P) -> impl Parser<I, Option<O>>
where
    I: Clone,
    P: Parser<I, O>,
{
    move |input: &mut I| {
        if condition {
            parser.parse(input).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub fn opt<I, O, P>(parser: P) -> impl Parser<I, Option<O>>
where
    I: Clone,
    P: Parser<I, O>,
{
    move |input: &mut I| match parser.parse_peek(input) {
        Ok(o) => Ok(Some(o)),
        Err(ParseError::Recoverable(_)) | Err(ParseError::Incomplete(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn repeat<I, O, P, C>(parser: P, count: C) -> impl Parser<I, Vec<O>>
where
    I: Clone,
    P: Parser<I, O>,
    C: Into<usize>,
{
    let count = count.into();
    move |input: &mut I| {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            res.push(parser.parse(input)?);
        }
        Ok(res)
    }
}

pub fn map_repeat<I, O, C, PC, P>(parse_count: PC, parser: P) -> impl Parser<I, Vec<O>>
where
    I: Clone,
    PC: Parser<I, C>,
    C: Into<usize>,
    P: Parser<I, O>,
{
    move |input: &mut I| {
        let count = parse_count.parse(input)?.into();
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            res.push(parser.parse(input)?);
        }
        Ok(res)
    }
}

/// Provides context to parse errors
pub fn context<I, O, P, C>(ctx: C, parser: P) -> impl Parser<I, O>
where
    I: Clone,
    P: Parser<I, O>,
    C: Clone + Into<ErrorContext>,
{
    move |input: &mut I| {
        let res = parser.parse(input);
        match res {
            Err(ParseError::Recoverable(_)) => Err(ParseError::Recoverable(ctx.clone().into())),
            Err(ParseError::Final(_)) => Err(ParseError::Final(ctx.clone().into())),
            Err(ParseError::Incomplete(n)) => Err(ParseError::Incomplete(n)),
            Ok(o) => Ok(o),
        }
    }
}
