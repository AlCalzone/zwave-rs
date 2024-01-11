use crate::{cc_api_assert_supported, expect_cc_or_timeout};
use crate::{CCAPIResult, CCInterviewContext, EndpointLike, CCAPI};
use zwave_cc::commandclass::{manufacturer_specific::*, CCAddressable};
use zwave_core::prelude::*;

pub struct ManufacturerSpecificCCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for ManufacturerSpecificCCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_version(&self) -> u8 {
        2
    }

    async fn interview<'ctx: 'a>(&self, ctx: &CCInterviewContext<'ctx>) -> CCAPIResult<()> {
        let endpoint = ctx.endpoint;

        ctx.log.info("interviewing Manufacturer Specific CC...");

        ctx.log.info("querying manufacturer information...");
        if let Some(response) = self.get().await? {
            println!(
                "received response for manufacturer information: {:?}",
                response
            );
        }

        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        // Nothing that requires refreshing
        Ok(())
    }
}

impl ManufacturerSpecificCCAPI<'_> {
    pub async fn get(&self) -> CCAPIResult<Option<ManufacturerSpecificCCReport>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = ManufacturerSpecificCCGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, ManufacturerSpecificCCReport);

        Ok(response)
    }

    pub fn supports_get_device_specific(&self) -> Option<bool> {
        self.endpoint.get_cc_version(self.cc_id()).map(|v| v >= 2)
    }

    pub async fn get_device_specific(
        &self,
        device_id_type: DeviceIdType,
    ) -> CCAPIResult<Option<Vec<u8>>> {
        cc_api_assert_supported!(self, get_device_specific);

        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = ManufacturerSpecificCCDeviceSpecificGet::builder()
            .device_id_type(device_id_type)
            .build()
            .with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, ManufacturerSpecificCCDeviceSpecificReport);

        Ok(response.map(|r| r.device_id))
    }
}
