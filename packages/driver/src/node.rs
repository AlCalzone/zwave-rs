use zwave_core::{definitions::NodeId, submodule};

submodule!(interview_stage);

#[derive(Debug)]
pub struct Node {
    id: NodeId,
    interview_stage: InterviewStage,
}

impl Node {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            interview_stage: InterviewStage::None,
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn interview_stage(&self) -> &InterviewStage {
        &self.interview_stage
    }

    pub fn set_interview_stage(&mut self, interview_stage: InterviewStage) {
        self.interview_stage = interview_stage;
    }
}
