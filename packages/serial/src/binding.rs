use crate::{error::Result, frame::RawSerialFrame};

#[cfg(feature = "std")]
pub trait SerialBinding {
    fn write(
        &mut self,
        frame: RawSerialFrame,
    ) -> impl core::future::Future<Output = Result<()>> + Send;

    fn read(&mut self) -> impl core::future::Future<Output = Option<RawSerialFrame>> + Send;
}

#[cfg(not(feature = "std"))]
pub trait SerialBinding {
    fn write(&mut self, frame: RawSerialFrame) -> impl core::future::Future<Output = Result<()>>;

    fn read(&mut self) -> impl core::future::Future<Output = Option<RawSerialFrame>>;
}
