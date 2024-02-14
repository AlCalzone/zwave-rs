extern crate bytes as bytes_crate;
use bytes_crate::BytesMut;

pub mod bytes;
pub mod combinators;
pub mod complete;
pub mod multi;
pub mod streaming;

mod branch;
pub use branch::*;
mod error;
pub use error::*;

pub trait Parser<O = ()> {
    /// Execute the parser on the input, advancing the input
    fn parse(&self, input: &mut BytesMut) -> ParseResult<O>;

    /// Execute the parser on the input, advancing the input only in case of success
    fn parse_peek(&self, input: &mut BytesMut) -> ParseResult<O> {
        let checkpoint = input.clone();
        let res = self.parse(input);
        if res.is_err() {
            *input = checkpoint;
        }
        res
    }
}

// Convenience implementation of Parser for functions
impl<O, F> Parser<O> for F
where
    F: Fn(&mut BytesMut) -> ParseResult<O>,
{
    fn parse(&self, input: &mut BytesMut) -> ParseResult<O> {
        self(input)
    }
}
