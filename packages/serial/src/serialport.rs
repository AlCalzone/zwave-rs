use crate::binding::SerialBinding;
use crate::error::*;
use crate::frame::RawSerialFrame;
use bytes::BytesMut;
use embedded_io::Error as _;
use embedded_io_async::{Read, Write};
use zwave_core::parse::Needed;
use zwave_core::prelude::*;

/// Codec that frames raw serial bytes into Z-Wave serial frames.
/// Works with any stream implementing `embedded_io_async` traits.
pub struct SerialCodec<S> {
    stream: S,
    read_buf: BytesMut,
}

impl<S: Read + Write> SerialCodec<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            read_buf: BytesMut::with_capacity(256),
        }
    }
}

impl<S: Read + Write + Unpin + Send> SerialBinding for SerialCodec<S> {
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        let mut buf = BytesMut::new();
        frame.serialize(&mut buf);
        self.stream
            .write_all(&buf)
            .await
            .map_err(|e| Error::Io(e.kind()))?;
        Ok(())
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        loop {
            match RawSerialFrame::parse_mut(&mut self.read_buf) {
                Ok(frame) => return Some(frame),
                Err(ParseError::Incomplete(needed)) => {
                    if let Needed::Size(n) = needed {
                        self.read_buf.reserve(n);
                    }
                    let mut tmp = [0u8; 256];
                    match self.stream.read(&mut tmp).await {
                        Ok(0) | Err(_) => return None,
                        Ok(n) => self.read_buf.extend_from_slice(&tmp[..n]),
                    }
                }
                Err(_) => {
                    if !self.read_buf.is_empty() {
                        let _ = self.read_buf.split_to(1);
                    }
                }
            }
        }
    }
}

/// Codec that frames raw serial bytes into Z-Wave serial frames.
/// Works with any stream implementing `futures::io` traits.
#[cfg(feature = "futures-io")]
pub struct FuturesSerialCodec<S> {
    stream: S,
    read_buf: BytesMut,
}

#[cfg(feature = "futures-io")]
impl<S: futures::io::AsyncRead + futures::io::AsyncWrite> FuturesSerialCodec<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            read_buf: BytesMut::with_capacity(256),
        }
    }
}

#[cfg(feature = "futures-io")]
impl<S: futures::io::AsyncRead + futures::io::AsyncWrite + Unpin + Send> SerialBinding
    for FuturesSerialCodec<S>
{
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        use futures::io::AsyncWriteExt;
        let mut buf = BytesMut::new();
        frame.serialize(&mut buf);
        self.stream
            .write_all(&buf)
            .await
            .map_err(Error::IO)?;
        Ok(())
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        use futures::io::AsyncReadExt;
        loop {
            match RawSerialFrame::parse_mut(&mut self.read_buf) {
                Ok(frame) => return Some(frame),
                Err(ParseError::Incomplete(needed)) => {
                    if let Needed::Size(n) = needed {
                        self.read_buf.reserve(n);
                    }
                    let mut tmp = [0u8; 256];
                    match self.stream.read(&mut tmp).await {
                        Ok(0) | Err(_) => return None,
                        Ok(n) => self.read_buf.extend_from_slice(&tmp[..n]),
                    }
                }
                Err(_) => {
                    if !self.read_buf.is_empty() {
                        let _ = self.read_buf.split_to(1);
                    }
                }
            }
        }
    }
}
