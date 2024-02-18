use super::{ensure_capacity, Serializable};
use bytes::BytesMut;

macro_rules! impl_int {
    ($un:ident, 1) => {
        paste::paste! {
            pub fn [<be_ $un>](val: $un) -> impl Serializable {
                use bytes::BufMut;
                move |output: &mut BytesMut| {
                    ensure_capacity(output, 1);
                    output.[<put_ $un>](val);
                }
            }
        }
    };
    ($un:ident, $bytes:literal) => {
        paste::paste! {
            pub fn [<be_ $un>](val: $un) -> impl Serializable {
                use bytes::BufMut;
                move |output: &mut BytesMut| {
                    ensure_capacity(output, $bytes);
                    output.[<put_ $un>](val);
                }
            }

            pub fn [<le_ $un>](val: $un) -> impl Serializable {
                use bytes::BufMut;
                move |output: &mut BytesMut| {
                    ensure_capacity(output, $bytes);
                    output.[<put_ $un _le>](val);
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

pub fn slice<S>(data: S) -> impl Serializable
where
    S: AsRef<[u8]>,
{
    move |output: &mut BytesMut| {
        let data = data.as_ref();
        ensure_capacity(output, data.len());
        output.extend_from_slice(data);
    }
}

pub fn empty(_: &mut BytesMut) {
    // Do nothing
}
