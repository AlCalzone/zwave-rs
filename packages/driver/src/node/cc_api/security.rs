use crate::expect_cc_or_timeout;
use crate::{CCAPIResult, EndpointLike, CCAPI};
use zwave_cc::commandclass::{security::*, CCAddressable};
use zwave_core::security::S0Nonce;
use zwave_core::prelude::*;

pub struct SecurityCCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for SecurityCCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security
    }

    fn cc_version(&self) -> u8 {
        1
    }

    fn interview_depends_on(&self) -> &'static [CommandClasses] {
        // Optional: Return a list of required CCs or remove this method
        &[ /* ... */]
    }

    async fn interview(&self) -> CCAPIResult<()> {
        let endpoint = self.endpoint;
        let node = endpoint.get_node();
        let cache = node.value_cache();
        let log = endpoint.logger();

        log.warn(|| "interviewing Security CC...");

        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        // Nothing that requires refreshing
        Ok(())
    }
}

impl SecurityCCAPI<'_> {
    pub async fn get_nonce(&self) -> CCAPIResult<Option<S0Nonce>> {
        // Optional: Test support for this command:
        // cc_api_assert_supported!(self, get);
        // and implement the supports_get() method using the zwccapisupp snippet

        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = SecurityCCNonceGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc.into(), None).await;
        let response = expect_cc_or_timeout!(response, SecurityCCNonceReport);

        Ok(response.map(|r| r.nonce))
    }
}
