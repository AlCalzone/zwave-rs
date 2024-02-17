use bytes::BytesMut;

use crate::encoding::BitOutput;

use super::ensure_capacity;
use super::Encoder;

pub fn bits<F>(f: F) -> impl Encoder
where
    F: Fn(&mut BitOutput),
{
    move |output: &mut BytesMut| {
        let mut bo = BitOutput::new();
        f(&mut bo);

        let data = bo.as_raw_slice();
        ensure_capacity(output, data.len());
        output.extend_from_slice(data);
    }
}
