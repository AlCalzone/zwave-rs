use crate::frame::RawSerialFrame;
use bytes::BytesMut;
use zwave_core::parse::Needed;
use zwave_core::prelude::*;

/// Reusable framing codec that buffers incoming bytes and extracts
/// Z-Wave serial frames. Platform-agnostic — callers feed it raw
/// bytes and consume parsed frames.
pub struct FrameCodec {
    read_buf: BytesMut,
}

impl FrameCodec {
    pub fn new() -> Self {
        Self {
            read_buf: BytesMut::with_capacity(256),
        }
    }

    /// Feed raw bytes into the codec's buffer.
    pub fn push_bytes(&mut self, data: &[u8]) {
        self.read_buf.extend_from_slice(data);
    }

    /// Try to extract the next complete frame from the buffer.
    /// Returns `None` if more data is needed.
    pub fn try_decode(&mut self) -> Option<RawSerialFrame> {
        loop {
            match RawSerialFrame::parse_mut(&mut self.read_buf) {
                Ok(frame) => return Some(frame),
                Err(ParseError::Incomplete(needed)) => {
                    if let Needed::Size(n) = needed {
                        self.read_buf.reserve(n);
                    }
                    return None;
                }
                Err(_) => {
                    // Garbage byte — discard and retry
                    if self.read_buf.is_empty() {
                        return None;
                    }
                    let _ = self.read_buf.split_to(1);
                }
            }
        }
    }
}
