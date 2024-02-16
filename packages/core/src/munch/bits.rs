use super::{combinators::map, Needed, ParseError, ParseResult, Parser};
use bytes::{Buf, Bytes};
use std::ops::{Add, Shl, Shr};

// Bit-level parsers operate on Bytes and a bit-offset.
// The bit-offset indicates the number of bits that have been already been consumed from the start of the input.
// It must be in the range [0..7] and always be relative to the first byte of the input.

/// Takes `count` bits from the input and interprets them as a big-endian unsigned integer of type `O`
pub fn take<O, C>(count: C) -> impl Parser<(Bytes, usize), O>
where
    O: From<u8> + Add<O, Output = O> + Shl<usize, Output = O> + Shr<usize, Output = O>,
    C: Into<usize>,
{
    let count: usize = count.into();
    move |(input, bit_offset): &mut (Bytes, usize)| {
        if count == 0 {
            return Ok(0u8.into());
        }

        let mut offset = *bit_offset;

        let needed_bytes = (count + offset) / 8;
        if input.remaining() < needed_bytes {
            return Err(ParseError::Incomplete(Needed::Size(
                needed_bytes - input.remaining(),
            )));
        }

        let mut ret: O = 0u8.into();
        let mut remaining = count;
        let mut skip_bytes: usize = 0;

        for byte in input.iter().take(needed_bytes) {
            // Discard all bits left of the offset
            let val: O = if offset == 0 {
                *byte
            } else {
                (*byte << offset) >> offset
            }
            .into();

            if remaining <= 8 - offset {
                // There are bits on the right we're not interested in, e.g.
                // remaining = 5, offset = 2
                // ..xxxxx.
                ret = (ret << remaining) + (val >> (8 - offset - remaining));
                offset += remaining;
            } else {
                // There are no remaining bits on the right, e.g.
                // remaining = 6, offset = 2
                // ..xxxxxx
                // or the remaining bits span multiple bytes, e.g.
                // remaining = 7, offset = 2
                // ..xxxxxx | x.......
                ret = (ret << (8 - offset)) + val;
                offset = 0;
                skip_bytes += 1;
            }

            remaining -= 8 - offset;
            if remaining == 0 {
                break;
            }
        }

        // Update the input bytes and offset
        input.advance(skip_bytes);
        *bit_offset = offset;

        Ok(ret)
    }
}

/// Wrapper around bit-level parsers to operate on Bytes.
/// Parsing starts at bit-offset 0 and discards partially consumed bytes.
pub fn bits<O, P>(parser: P) -> impl Parser<Bytes, O>
where
    P: Parser<(Bytes, usize), O>,
{
    move |input: &mut Bytes| {
        let mut bit_input = (input.clone(), 0usize);

        let ret = parser.parse(&mut bit_input);

        let (mut bit_input, offset) = bit_input;
        if offset > 0 {
            bit_input.advance(1);
        }
        *input = bit_input;
        ret
    }
}

pub fn bool(input: &mut (Bytes, usize)) -> ParseResult<bool> {
    map(take(1usize), |x: u8| x != 0).parse(input)
}
