use crate::{CCAPIResult, CCInterviewContext, EndpointLike, CCAPI};
use zwave_core::prelude::*;

pub struct Crc16CCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for Crc16CCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::CRC16Encapsulation
    }

    fn cc_version(&self) -> u8 {
        1
    }

    async fn interview<'ctx: 'a>(&self, _ctx: &CCInterviewContext<'ctx>) -> CCAPIResult<()> {
        // Nothing to do
        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        // Nothing that requires refreshing
        Ok(())
    }
}

impl Crc16CCAPI<'_> {}
