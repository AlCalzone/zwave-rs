use super::{ParseError, Parser};
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
            return Err(ParseError::recoverable(()));
        }
        Ok(output)
    }
}
