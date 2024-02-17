use super::{combinators::map_parser, ParseError, ParseResult, Parser, ToLength};
use bytes::Bytes;

pub fn many_0<I, O, P>(parser: P) -> impl Parser<I, Vec<O>>
where
    I: Clone,
    P: Parser<I, O>,
{
    move |input: &mut I| {
        let mut output = Vec::new();
        while let Ok(o) = parser.parse_peek(input) {
            output.push(o);
        }
        Ok(output)
    }
}

pub fn many_n<I, O, P>(at_least: usize, parser: P) -> impl Parser<I, Vec<O>>
where
    I: Clone,
    P: Parser<I, O>,
{
    move |input: &mut I| {
        let checkpoint = input.clone();
        let mut output = Vec::new();
        while let Ok(o) = parser.parse_peek(input) {
            output.push(o);
        }
        if output.len() < at_least {
            *input = checkpoint;
            return Err(ParseError::recoverable(()));
        }
        Ok(output)
    }
}

pub fn length_data<N, P>(length_parser: P) -> impl Parser<Bytes, Bytes>
where
    P: Parser<Bytes, N>,
    N: ToLength,
{
    move |input: &mut Bytes| {
        let length = length_parser.parse(input)?.to_length();
        super::bytes::complete::take(length).parse(input)
    }
}

pub fn length_value<O, N, P, PV>(length_parser: P, value_parser: PV) -> impl Parser<Bytes, O>
where
    P: Parser<Bytes, N>,
    N: ToLength,
    PV: Parser<Bytes, O>,
{
    map_parser(length_data(length_parser), value_parser)
}

macro_rules! impl_parser_for_tuple {
    ($($idx:literal),+) => {
        paste::paste! {
            impl<I, $([<P $idx>], [<O $idx>]),+> Parser<I, ($([<O $idx>]),+,)> for ($([<P $idx>]),+,)
            where
                I: Clone,
            $(
                [<P $idx>]: Parser<I, [<O $idx>]>,
            )+
            {
                fn parse(&self, input: &mut I) -> ParseResult<($([<O $idx>]),+,)> {
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

pub fn separated_pair<I, O1, OS, O2, P1, PS, P2>(
    first: P1,
    sep: PS,
    second: P2,
) -> impl Parser<I, (O1, O2)>
where
    I: Clone,
    P1: Parser<I, O1>,
    PS: Parser<I, OS>,
    P2: Parser<I, O2>,
{
    move |input: &mut I| {
        let o1 = first.parse(input)?;
        let _ = sep.parse(input)?;
        let o2 = second.parse(input)?;
        Ok((o1, o2))
    }
}
