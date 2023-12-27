use crate::{error::*, Driver, Ready};

impl Driver<Ready> {
    pub async fn interview_nodes(&self) -> Result<()> {
        for node in self.nodes() {
            node.interview().await?;
        }

        Ok(())
    }
}
