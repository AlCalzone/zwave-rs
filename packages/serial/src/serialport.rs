use crate::binding::SerialBinding;
use crate::error::*;
use crate::frame::RawSerialFrame;
use bytes::BytesMut;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Encoder, Framed};
use zwave_core::parse::Needed;
use zwave_core::prelude::*;

pub enum ZWavePort {
    Serial(SerialPort),
    Tcp(TcpSocket),
}

impl SerialBinding for ZWavePort {
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        match self {
            ZWavePort::Serial(port) => port.write(frame).await,
            ZWavePort::Tcp(port) => port.write(frame).await,
        }
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        match self {
            ZWavePort::Serial(port) => port.read().await,
            ZWavePort::Tcp(port) => port.read().await,
        }
    }
}

// FIXME: The only difference between SerialPort and TcpSocket is the type of the stream
// We should unify them into a single struct with a generic type parameter

pub struct SerialPort {
    writer: SplitSink<Framed<SerialStream, SerialFrameCodec>, RawSerialFrame>,
    reader: SplitStream<Framed<SerialStream, SerialFrameCodec>>,
}

impl SerialPort {
    pub fn new(path: &str) -> Result<Self> {
        #[allow(unused_mut)]
        let mut port = tokio_serial::new(path, 115_200).open_native_async()?;

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");
        let codec = SerialFrameCodec.framed(port);
        let (writer, reader) = codec.split();
        Ok(Self { writer, reader })
    }
}

impl SerialBinding for SerialPort {
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        self.writer.send(frame).await?;
        Ok(())
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        let ret = self.reader.next().await;
        match ret {
            Some(Ok(frame)) => Some(frame),
            _ => None,
        }
    }
}

pub struct TcpSocket {
    writer: SplitSink<Framed<TcpStream, SerialFrameCodec>, RawSerialFrame>,
    reader: SplitStream<Framed<TcpStream, SerialFrameCodec>>,
}

impl TcpSocket {
    pub fn new(addr: &str) -> Result<Self> {
        let stream = std::net::TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        let stream = TcpStream::from_std(stream)?;

        let codec = SerialFrameCodec.framed(stream);
        let (writer, reader) = codec.split();
        Ok(Self { writer, reader })
    }
}

impl SerialBinding for TcpSocket {
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        self.writer.send(frame).await?;
        Ok(())
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        let ret = self.reader.next().await;
        match ret {
            Some(Ok(frame)) => Some(frame),
            _ => None,
        }
    }
}

struct SerialFrameCodec;

impl Decoder for SerialFrameCodec {
    type Item = RawSerialFrame;
    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match RawSerialFrame::parse_mut(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(ParseError::Incomplete(n)) => {
                // When expecting more bytes, reserve space for them
                if let Needed::Size(n) = n {
                    src.reserve(n);
                }
                Ok(None)
            }
            Err(_) => {
                // There was a problem parsing the frame, but the serial port doesn't care about that
                Ok(None)
            }
        }
    }
}

impl Encoder<RawSerialFrame> for SerialFrameCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: RawSerialFrame,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        item.serialize(dst);
        Ok(())
    }
}
