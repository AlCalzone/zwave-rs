use crate::binding::SerialBinding;
use crate::error::{IntoResult, Result};
use crate::frame::RawSerialFrame;
use bytes::{Buf, BytesMut};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct SerialPort {
    writer: SplitSink<Framed<SerialStream, SerialFrameCodec>, RawSerialFrame>,
    reader: SplitStream<Framed<SerialStream, SerialFrameCodec>>,
}

impl SerialBinding for SerialPort {
    fn new(path: &str) -> Result<Self> {
        let mut port = tokio_serial::new(path, 115_200).open_native_async()?;

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");
        let codec = SerialFrameCodec.framed(port);
        let (writer, reader) = codec.split();
        Ok(Self { writer, reader })
    }

    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        match &frame {
            RawSerialFrame::Data(data) => {
                println!(">> {}", hex::encode(&data));
            }
            RawSerialFrame::ACK | RawSerialFrame::CAN | RawSerialFrame::NAK => {
                println!(">> {:?}", &frame);
            }
            _ => (),
        }

        self.writer.send(frame).await
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
    type Error = crate::error::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match RawSerialFrame::parse(src) {
            Ok((remaining, frame)) => {
                let bytes_read = src.len() - remaining.len();
                src.advance(bytes_read);
                Ok(Some(frame))
            }
            Err(nom::Err::Incomplete(_)) => Ok(None),
            e => e.into_result().map(|_| None),
        }
    }
}

impl Encoder<RawSerialFrame> for SerialFrameCodec {
    type Error = crate::error::Error;

    fn encode(
        &mut self,
        item: RawSerialFrame,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        let data: Vec<u8> = (&item).try_into()?;
        dst.extend_from_slice(data.as_slice());
        Ok(())
    }
}
