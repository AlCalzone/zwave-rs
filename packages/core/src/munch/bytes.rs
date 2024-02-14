use crate::bake::Encoder;

use super::{Needed, ParseError, Parser};
use bytes::{Buf, Bytes};

macro_rules! impl_int {
    ($un:ident, 1) => {
        paste::paste! {
            pub fn [<be_ $un>]() -> impl Parser<$un> {
                move |input: &mut Bytes| {
                    if input.remaining() < 1 {
                        Err(ParseError::Incomplete(Needed::Size(1)))
                    } else {
                        Ok(input.[<get_ $un>]())
                    }
                }
            }
        }
    };
    ($un:ident, $bytes:literal) => {
        paste::paste! {
            pub fn [<be_ $un>]() -> impl Parser<$un> {
                move |input: &mut Bytes| {
                    if input.remaining() < $bytes {
                        Err(ParseError::Incomplete(Needed::Size($bytes)))
                    } else {
                        Ok(input.[<get_ $un>]())
                    }
                }
            }

            pub fn [<le_ $un>]() -> impl Parser<$un> {
                move |input: &mut Bytes| {
                    if input.remaining() < $bytes {
                        Err(ParseError::Incomplete(Needed::Size($bytes)))
                    } else {
                        Ok(input.[<get_ $un _le>]())
                    }
                }
            }
        }
    };
}

impl_int!(u8, 1);
impl_int!(u16, 2);
impl_int!(u32, 4);
impl_int!(u64, 8);
impl_int!(i8, 1);
impl_int!(i16, 2);
impl_int!(i32, 4);
impl_int!(i64, 8);
