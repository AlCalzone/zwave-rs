use super::{ParseError, Parser};
use bytes::Bytes;

pub fn many_0<O, P>(parser: P) -> impl Parser<Vec<O>>
where
    P: Parser<O>,
{
    move |input: &mut Bytes| {
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
    move |input: &mut Bytes| {
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
