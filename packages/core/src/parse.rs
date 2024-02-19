extern crate bytes as bytes_crate;
use bytes_crate::Bytes;

pub mod bits;
pub mod bytes;
pub mod combinators;
pub mod multi;

mod branch;
pub use branch::*;
mod error;
pub use error::*;

pub trait Parsable
where
    Self: Sized,
{
    fn parse(i: &mut Bytes) -> ParseResult<Self>;
}

pub trait BitParsable
where
    Self: Sized,
{
    fn parse(i: &mut (Bytes, usize)) -> crate::parse::ParseResult<Self>;
}

pub trait Parser<I: Clone, O = Self> {
    /// Execute the parser on the input, advancing the input
    fn parse(&self, input: &mut I) -> ParseResult<O>;

    /// Execute the parser on the input, advancing the input only in case of success
    fn parse_peek(&self, input: &mut I) -> ParseResult<O> {
        let checkpoint = input.clone();
        let res = self.parse(input);
        if res.is_err() {
            *input = checkpoint;
        }
        res
    }
}

// Convenience implementation of Parser for functions
impl<I, O, F> Parser<I, O> for F
where
    I: Clone,
    F: Fn(&mut I) -> ParseResult<O>,
{
    fn parse(&self, input: &mut I) -> ParseResult<O> {
        self(input)
    }
}

pub trait ToLength {
    fn to_length(&self) -> usize;
}

impl ToLength for u8 {
    fn to_length(&self) -> usize {
        *self as usize
    }
}

impl ToLength for u16 {
    fn to_length(&self) -> usize {
        *self as usize
    }
}

impl ToLength for u32 {
    fn to_length(&self) -> usize {
        *self as usize
    }
}

impl ToLength for u64 {
    fn to_length(&self) -> usize {
        *self as usize
    }
}
