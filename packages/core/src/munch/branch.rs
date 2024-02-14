use super::{ParseResult, Parser};
use bytes::Bytes;

pub trait Alt<O> {
    fn choice(&self, input: &mut Bytes) -> ParseResult<O>;
}

macro_rules! impl_alt_trait {
    ($($idx:literal),+; $last:tt) => {
        paste::paste! {
            impl<$([<P $idx>]),+, [<P $last>], O> Alt<O> for ($([<P $idx>]),+, [<P $last>])
            where
            $(
                [<P $idx>]: Parser<O>,
            )+
                [<P $last>]: Parser<O>,
            {
                fn choice(&self, input: &mut Bytes) -> ParseResult<O> {
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
            impl<[<P $zero>], O> Alt<O> for ([<P $zero>],)
            where
                [<P $zero>]: Parser<O>,
            {
                fn choice(&self, input: &mut Bytes) -> ParseResult<O> {
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
            impl<$([<P $idx>], [<O $idx>]),+> Parser<($([<O $idx>]),+,)> for ($([<P $idx>]),+,)
            where
            $(
                [<P $idx>]: Parser<[<O $idx>]>,
            )+
            {
                fn parse(&self, input: &mut Bytes) -> ParseResult<($([<O $idx>]),+,)> {
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
