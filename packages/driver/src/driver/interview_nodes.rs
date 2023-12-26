use crate::{error::*, Driver, Ready};
use zwave_core::definitions::*;

impl Driver<Ready> {
    pub async fn interview_nodes(&self) -> Result<()> {
        for node in self.nodes() {
            node.interview().await?;
        }

        Ok(())
    }
}
