use crate::expect_cc_or_timeout;
use crate::{CCAPIResult, EndpointLike, CCAPI};
use zwave_cc::commandclass::{binary_switch::*, CCAddressable};
use zwave_core::prelude::*;

pub struct BinarySwitchCCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for BinarySwitchCCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::BinarySwitch
    }

    fn cc_version(&self) -> u8 {
        2
    }

    async fn interview(&self) -> CCAPIResult<()> {
        let log = self.endpoint.logger();

        log.info(|| "interviewing Binary Switch CC...");

        // Try to query the current state
        self.refresh_values().await?;

        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        let log = self.endpoint.logger();

        log.info(|| "quering Binary Switch state...");

        if let Some(response) = self.get().await? {
            log.info(|| format!("received Binary Switch CC state: {:?}", response));
        }

        Ok(())
    }
}

impl BinarySwitchCCAPI<'_> {
    pub async fn get(&self) -> CCAPIResult<Option<BinarySwitchCCReport>> {
        // Test support for this command:
        // cc_api_assert_supported!(self, get);
        // and implement the supports_get() method using the zwccapisupp snippet
        // FIXME: get is only supported in singlecast

        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = BinarySwitchCCGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc.into(), None).await;
        let response = expect_cc_or_timeout!(response, BinarySwitchCCReport);

        Ok(response)
    }

    pub async fn set(&self, value: BinarySet, duration: Option<DurationSet>) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = BinarySwitchCCSet::builder()
            .target_value(value)
            .duration(duration)
            .build()
            .with_destination(node.id().into());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }
}
