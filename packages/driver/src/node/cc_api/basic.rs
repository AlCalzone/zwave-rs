use crate::{CCInterviewContext, Endpoint, ExecNodeCommandResult, CCAPI};
use zwave_cc::commandclass::{BasicCCSet, CCAddressable};
use zwave_core::prelude::*;

pub struct BasicCCAPI<'a> {
    endpoint: &'a dyn Endpoint,
}

impl<'a> CCAPI<'a> for BasicCCAPI<'a> {
    fn new(endpoint: &'a dyn Endpoint) -> Self
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

    async fn interview(&self, ctx: &CCInterviewContext<'_>) {
        println!(
            "Node {}, {} - Interviewing Basic CC",
            ctx.endpoint.node_id(),
            ctx.endpoint.index()
        );
        // ...
    }
}

impl BasicCCAPI<'_> {
    pub async fn set(&self, value: LevelSet) -> ExecNodeCommandResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = BasicCCSet::builder()
            .target_value(value)
            .build()
            .with_destination(node.id().into());
        driver.exec_node_command(&cc, None).await?;
        Ok(())
    }
}
