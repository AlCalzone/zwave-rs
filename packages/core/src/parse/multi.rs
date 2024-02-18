use super::{
    bytes::{
        be_u8,
        complete::{literal, take},
    },
    combinators::map_parser,
    ToLength,
};
use crate::prelude::*;
use bitvec::prelude::*;
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

/// Parses a bitmask into a `Vec<u8>`. The least significant bit is mapped to `bit0_value`. The first byte is considerd to be the bitmask length.
pub fn variable_length_bitmask_u8(i: &mut Bytes, bit0_value: u8) -> ParseResult<Vec<u8>> {
    let bitmask = length_data(be_u8).parse(i)?;

    let view = bitmask.view_bits::<Lsb0>();
    let ret = view
        .iter_ones()
        .map(|index| (index as u8) + bit0_value)
        .collect::<Vec<_>>();
    Ok(ret)
}

/// Parses a bitmask with the given length into a `Vec<u8>`. The least significant bit is mapped to `bit0_value`.
pub fn fixed_length_bitmask_u8(
    i: &mut Bytes,
    bit0_value: u8,
    bitmask_len: usize,
) -> ParseResult<Vec<u8>> {
    let bitmask = take(bitmask_len).parse(i)?;

    let view = bitmask.view_bits::<Lsb0>();
    let ret = view
        .iter_ones()
        .map(|index| (index as u8) + bit0_value)
        .collect::<Vec<_>>();
    Ok(ret)
}

/// Parses a list of supported and controlled CCs that starts with a length byte
pub fn variable_length_cc_list(
    i: &mut Bytes,
) -> ParseResult<(
    Vec<CommandClasses>, // supported
    Vec<CommandClasses>, // controlled
)> {
    map_parser(
        length_data(be_u8),
        separated_pair(
            many_0(CommandClasses::parse),
            literal(COMMAND_CLASS_SUPPORT_CONTROL_MARK),
            many_0(CommandClasses::parse),
        ),
    )
    .parse(i)
}

/// Parses a list of supported and controlled CCs with the given length
pub fn fixed_length_cc_list(
    i: &mut Bytes,
    len: usize,
) -> ParseResult<(
    Vec<CommandClasses>, // supported
    Vec<CommandClasses>, // controlled
)> {
    map_parser(
        take(len),
        separated_pair(
            many_0(CommandClasses::parse),
            literal(COMMAND_CLASS_SUPPORT_CONTROL_MARK),
            many_0(CommandClasses::parse),
        ),
    )
    .parse(i)
}

/// Parses a list of supported (NOT controlled) CCs with the given length
pub fn fixed_length_cc_list_only_supported(
    i: &mut Bytes,
    len: usize,
) -> ParseResult<Vec<CommandClasses>> {
    map_parser(take(len), many_0(CommandClasses::parse)).parse(i)
}
