extern crate bytes as bytes_crate;
use bitvec::prelude::*;
use bytes_crate::{BufMut, Bytes, BytesMut};

pub mod bits;
pub mod bytes;
pub mod sequence;

pub const DEFAULT_CAPACITY: usize = 64;
const CAPACITY_INCREMENT: usize = 32;

pub trait Serializable {
    /// Write the value into the given buffer
    fn serialize(&self, output: &mut BytesMut);

    fn as_bytes_mut(&self) -> BytesMut {
        let mut output = BytesMut::with_capacity(DEFAULT_CAPACITY);
        self.serialize(&mut output);
        output
    }

    fn as_bytes(&self) -> Bytes {
        self.as_bytes_mut().freeze()
    }
}

pub type BitOutput = BitVec<u8, Msb0>;

pub trait BitSerializable {
    fn write(&self, b: &mut BitOutput);
}

// Convenience implementation of Serializable for functions
impl<F> Serializable for F
where
    F: Fn(&mut BytesMut),
{
    fn serialize(&self, output: &mut BytesMut) {
        self(output)
    }
}

// Convenience implementation of Serializable for Option<Serializable>
impl<T> Serializable for Option<T>
where
    T: Serializable,
{
    fn serialize(&self, output: &mut BytesMut) {
        if let Some(v) = self {
            v.serialize(output);
        }
    }
}

pub trait SerializableWith<Context> {
    /// Write the value into the given buffer
    fn serialize(&self, output: &mut BytesMut, ctx: Context);

    fn as_bytes_mut(&self, ctx: Context) -> BytesMut {
        let mut output = BytesMut::with_capacity(DEFAULT_CAPACITY);
        self.serialize(&mut output, ctx);
        output
    }

    fn as_bytes(&self, ctx: Context) -> Bytes {
        self.as_bytes_mut(ctx).freeze()
    }
}

// NOTE on BytesMut usage:
// One key difference from Vec<u8> is that most operations do not implicitly grow the buffer. This
// means that calling my_bytes.put("hello world"); could panic if my_bytes does not have enough capacity.
// Before writing to the buffer, ensure that there is enough remaining capacity by calling my_bytes.remaining_mut().
// In general, avoiding calls to reserve is preferable.

/// Ensures that the given buffer has enough remaining capacity to write the given number of bytes.
#[inline(always)]
pub(crate) fn ensure_capacity(output: &mut BytesMut, required: usize) {
    if output.remaining_mut() < required {
        // We do not want to re-allocate often, so we reserve a bit more than needed.
        // Z-Wave frames should usually fit into the initial buffer size of 64 bytes.
        // If not, they will rarely exceed it by a lot, so we'll resize the buffer in
        // 32 byte increments.
        let mut additional = CAPACITY_INCREMENT;
        while additional < required {
            additional += CAPACITY_INCREMENT;
        }
        output.reserve(additional);
    }
}
