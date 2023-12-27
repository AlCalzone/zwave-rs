use zwave_core::{definitions::*, submodule};

use crate::{error::Result, Driver, Ready};

submodule!(interview_stage);
submodule!(storage);

macro_rules! read {
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .driver
            .get_node_storage($node_id)
            .map(|storage| (*storage).$field)
    };
}

macro_rules! read_locked {
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .driver
            .get_node_storage($node_id)
            .map(|storage| *storage.$field.read().unwrap())
    };
}

macro_rules! write_locked {
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .driver
            .get_node_storage($node_id)
            .map(|storage| storage.$field.write().unwrap())
    };
}

// macro_rules! read_atomic {
//     ($self:ident, $field:ident) => {
//         read!($self, $field).load(Ordering::Relaxed)
//     };
// }

// macro_rules! write_atomic {
//     ($self:ident, $field:ident, $value:expr) => {
//         read!($self, $field).store($value, Ordering::Relaxed);
//     };
// }

pub struct Node<'a> {
    id: NodeId,
    protocol_data: NodeInformationProtocolData,
    driver: &'a Driver<Ready>,
}

impl<'a> Node<'a> {
    pub fn new(
        id: NodeId,
        protocol_data: NodeInformationProtocolData,
        driver: &'a Driver<Ready>,
    ) -> Self {
        Self {
            id,
            protocol_data,
            driver,
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn interview_stage(&self) -> InterviewStage {
        read_locked!(self, &self.id, interview_stage).unwrap_or(InterviewStage::None)
    }

    pub fn set_interview_stage(&self, interview_stage: InterviewStage) {
        if let Some(mut handle) = write_locked!(self, &self.id, interview_stage) {
            *handle = interview_stage;
        }
    }

    pub fn protocol_data(&self) -> &NodeInformationProtocolData {
        &self.protocol_data
    }

    pub fn can_sleep(&self) -> bool {
        !self.protocol_data.listening && self.protocol_data.frequent_listening.is_none()
    }

    pub async fn interview(&self) -> Result<()> {
        // if self.interview_stage() == InterviewStage::None {
        //     self.set_interview_stage(InterviewStage::ProtocolInfo);

        //     let protocol_info = self.driver.get_node_protocol_info(&self.id, None).await?;
        //     println!("Node {:?} protocol info: {:?}", &self.id, protocol_info);
        // }

        Ok(())
    }
}
