use crate::binding::SerialBinding;
use crate::error::*;
use crate::frame::RawSerialFrame;
use bytes::{Buf, BufMut, BytesMut};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Encoder, Framed};
use zwave_core::encoding;
use zwave_core::munch::{Needed, ParseError};
use zwave_core::prelude::*;

pub struct SerialPort {
    writer: SplitSink<Framed<SerialStream, SerialFrameCodec>, RawSerialFrame>,
    reader: SplitStream<Framed<SerialStream, SerialFrameCodec>>,
}

impl SerialBinding for SerialPort {
    fn new(path: &str) -> Result<Self> {
        #[allow(unused_mut)]
        let mut port = tokio_serial::new(path, 115_200).open_native_async()?;

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");
        let codec = SerialFrameCodec.framed(port);
        let (writer, reader) = codec.split();
        Ok(Self { writer, reader })
    }

    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        // Not sure why, but doing this exects EncodingError to implement From<io::Error>,
        // although we'd actually want our local error type to be used.
        // TODO: Fix this at some point
        self.writer.send(frame).await.map_err(|e| match e {
            EncodingError::Parse(_) => {
                todo!("A parse error should not occur when sending data to the serial port")
            }
            EncodingError::NotImplemented(_) => {
                todo!(
                    "A not implemented error should not occur when sending data to the serial port"
                )
            }
            EncodingError::Serialize(reason) => std::io::Error::other(reason).into(),
        })
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
    type Error = encoding::EncodingError;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match RawSerialFrame::parse_mut(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(ParseError::Incomplete(n)) => {
                if let Needed::Size(n) = n {
                    src.reserve(n);
                }
                Ok(None)
            }
            Err(_) => Err(EncodingError::Parse(None)),
        }
    }
}

impl Encoder<RawSerialFrame> for SerialFrameCodec {
    type Error = encoding::EncodingError;

    fn encode(
        &mut self,
        item: RawSerialFrame,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        use zwave_core::bake::Encoder;
        item.write(dst);
        Ok(())
    }
}
