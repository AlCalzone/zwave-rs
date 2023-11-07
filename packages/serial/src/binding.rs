use crate::{error::Result, frame::SerialFrame};

pub type SerialListener = crossbeam_channel::Receiver<SerialFrame>;
pub trait SerialWriter {
    // FIXME: Do not accept garbage here
    fn write(&self, frame: SerialFrame) -> Result<()>;
    fn write_raw(&self, data: &[u8]) -> Result<()>;
}

pub trait Binding {
    type Open;

    fn new(path: &str) -> Self;
    fn open(self) -> Result<Self::Open>;
}

pub trait OpenBinding {
    type Closed;

    fn close(self) -> Result<Self::Closed>;
    fn listener(&self) -> SerialListener;
    fn writer(&self) -> Box<dyn SerialWriter>;
}
