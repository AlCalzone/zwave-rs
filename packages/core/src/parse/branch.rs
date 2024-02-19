use super::{ParseResult, Parser};

pub trait Alt<I, O> {
    fn choice(&self, input: &mut I) -> ParseResult<O>;
}

macro_rules! impl_alt_trait {
    ($($idx:literal),+; $last:tt) => {
        paste::paste! {
            impl<I, $([<P $idx>]),+, [<P $last>], O> Alt<I, O> for ($([<P $idx>]),+, [<P $last>])
            where
                I: Clone,
            $(
                [<P $idx>]: Parser<I, O>,
            )+
                [<P $last>]: Parser<I, O>,
            {
                fn choice(&self, input: &mut I) -> ParseResult<O> {
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
            impl<I, [<P $zero>], O> Alt<I, O> for ([<P $zero>],)
            where
                I: Clone,
                [<P $zero>]: Parser<I, O>,
            {
                fn choice(&self, input: &mut I) -> ParseResult<O> {
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
