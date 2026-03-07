use crate::binding::SerialBinding;
use crate::error::*;
use crate::frame::RawSerialFrame;
use asynchronous_codec::{Decoder, Encoder, Framed};
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use zwave_core::parse::Needed;
use zwave_core::prelude::*;

pub struct FramedBinding<S> {
    writer: SplitSink<Framed<S, SerialFrameCodec>, RawSerialFrame>,
    reader: SplitStream<Framed<S, SerialFrameCodec>>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> FramedBinding<S> {
    pub fn new(stream: S) -> Self {
        let framed = Framed::new(stream, SerialFrameCodec);
        let (writer, reader) = framed.split();
        Self { writer, reader }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> SerialBinding for FramedBinding<S> {
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        self.writer.send(frame).await?;
        Ok(())
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        match self.reader.next().await {
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

impl Encoder for SerialFrameCodec {
    type Item<'a> = RawSerialFrame;
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: Self::Item<'_>,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        item.serialize(dst);
        Ok(())
    }
}
