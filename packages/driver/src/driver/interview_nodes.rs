use crate::{driver_api::DriverApi, error::*, Ready};

impl DriverApi<Ready> {
    pub async fn interview_nodes(&self) -> Result<()> {
        for node in self.nodes() {
            node.interview().await?;
        }

        Ok(())
    }
}
