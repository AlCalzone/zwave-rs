use crate::{error::Result, frame::RawSerialFrame};

pub trait SerialBinding {
    fn write(
        &mut self,
        frame: RawSerialFrame,
    ) -> impl core::future::Future<Output = Result<()>>;

    fn read(&mut self) -> impl core::future::Future<Output = Option<RawSerialFrame>>;
}
