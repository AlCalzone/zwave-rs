use crate::{error::Result, frame::RawSerialFrame};

pub trait SerialBinding {
    fn write(
        &mut self,
        frame: RawSerialFrame,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn read(&mut self) -> impl std::future::Future<Output = Option<RawSerialFrame>> + Send;
}
