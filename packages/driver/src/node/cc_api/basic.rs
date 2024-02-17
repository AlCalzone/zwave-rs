use crate::{expect_cc_or_timeout, CCAPIResult, CCInterviewContext, EndpointLike, CCAPI};
use zwave_cc::commandclass::{basic::*, CCAddressable};
use zwave_core::{cache::CacheExt, prelude::*};

pub struct BasicCCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for BasicCCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Basic
    }

    fn cc_version(&self) -> u8 {
        2
    }

    async fn interview<'ctx: 'a>(&self, ctx: &CCInterviewContext<'ctx>) -> CCAPIResult<()> {
        let endpoint = ctx.endpoint;
        let node = endpoint.get_node();
        let cache = node.value_cache();

        ctx.log.info(||"interviewing Basic CC...");

        // Try to query the current state
        self.refresh_values().await?;

        // Remove Basic CC support when there was no response
        if cache
            .read_level_report(&BasicCCValues::current_value().id)
            .is_none()
        {
            ctx.log.info(||
                "No response to Basic Get command, assuming the node does not support Basic CC...",
            );

            // TODO: Actually remove Basic CC support
        }

        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        println!("Quering Basic CC state...");

        if let Some(basic_response) = self.get().await? {
            println!("received Basic CC state: {:?}", basic_response);
        }

        Ok(())
    }
}

impl BasicCCAPI<'_> {
    pub async fn set(&self, value: LevelSet) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = BasicCCSet::builder()
            .target_value(value)
            .build()
            .with_destination(node.id().into());
        driver.exec_node_command(&cc, None).await?;
        Ok(())
    }

    pub async fn get(&self) -> CCAPIResult<Option<BasicCCReport>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = BasicCCGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, BasicCCReport);

        Ok(response)
    }
}
