use zwave_core::{definitions::NodeId, submodule};

use crate::{error::Result, Driver, Ready};

submodule!(interview_stage);
submodule!(storage);

pub struct Node<'a> {
    id: NodeId,
    driver: &'a Driver<Ready>,
}

impl<'a> Node<'a> {
    pub fn new(id: NodeId, driver: &'a Driver<Ready>) -> Self {
        Self { id, driver }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn interview_stage(&self) -> InterviewStage {
        self.driver
            .get_node_interview_stage(&self.id)
            .unwrap_or(InterviewStage::None)
    }

    pub fn set_interview_stage(&self, interview_stage: InterviewStage) {
        self.driver
            .set_node_interview_stage(&self.id, interview_stage);
    }

    pub async fn interview(&self) -> Result<()> {
        if self.interview_stage() == InterviewStage::None {
            self.set_interview_stage(InterviewStage::ProtocolInfo);

            let protocol_info = self.driver.get_node_protocol_info(&self.id, None).await?;
            println!("Node {:?} protocol info: {:?}", &self.id, protocol_info);
        }

        Ok(())
    }
}
