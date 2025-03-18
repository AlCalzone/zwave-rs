use crate::{driver_api::DriverApi, error::*, Ready};

// FIXME: We should have a wrapper to expose only supported commands to lib users

impl DriverApi {
    // FIXME: Assert that the driver is in the Ready state

    pub async fn interview_nodes(&self) -> Result<()> {
        for node in self.nodes() {
            node.interview().await?;
        }

        Ok(())
    }
}
