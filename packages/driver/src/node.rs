use zwave_cc::commandclass::{CCAddressable, NoOperationCC};
use zwave_core::{definitions::*, submodule};

use crate::{
    ControllerCommandResult, Driver,
    ExecNodeCommandError, Ready,
};

submodule!(interview);
submodule!(storage);

macro_rules! read {
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .driver
            .get_node_storage($node_id)
            .map(|storage| storage.$field)
    };
}

macro_rules! read_locked {
    ($self:ident, $node_id:expr, $field:ident; deref) => {
        $self
            .driver
            .get_node_storage($node_id)
            .map(|storage| *storage.$field.read().unwrap())
    };
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .driver
            .get_node_storage($node_id)
            .map(|storage| storage.$field.read().unwrap())
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

macro_rules! read_endpoint_locked {
    ($self:ident, $node_id:expr, $endpoint_index:expr, $field:ident; deref) => {
        $self
            .driver
            .get_endpoint_storage($node_id, $endpoint_index)
            .map(|storage| *storage.$field.read().unwrap())
    };
    ($self:ident, $node_id:expr, $endpoint_index:expr, $field:ident) => {
        $self
            .driver
            .get_endpoint_storage($node_id, $endpoint_index)
            .map(|storage| storage.$field.read().unwrap())
    };
}

macro_rules! write_endpoint_locked {
    ($self:ident, $node_id:expr, $endpoint_index:expr, $field:ident) => {
        $self
            .driver
            .get_endpoint_storage($node_id, $endpoint_index)
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

// FIXME: We probably want a struct with this name, so this needs a rename
pub trait Endpoint {
    fn node_id(&self) -> NodeId;
    fn get_node(&self) -> &Node<'_>;
    fn index(&self) -> EndpointIndex;

    // TODO: Add the rest
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
        read_locked!(self, &self.id, interview_stage; deref).unwrap_or(InterviewStage::None)
    }

    pub fn set_interview_stage(&self, interview_stage: InterviewStage) {
        if let Some(mut handle) = write_locked!(self, &self.id, interview_stage) {
            *handle = interview_stage;
        }
    }

    pub fn protocol_data(&self) -> &NodeInformationProtocolData {
        &self.protocol_data
    }

    pub fn supported_command_classes(&self) -> Vec<CommandClasses> {
        read_endpoint_locked!(self, &self.id, &self.index(), supported_command_classes)
            .map(|ccs| ccs.clone())
            .unwrap_or_default()
    }

    // FIXME: We need an easier way to modify CC support on nodes and endpoints -> Separate trait!

    pub fn set_supported_command_classes(&self, ccs: Vec<CommandClasses>) {
        if let Some(mut handle) =
            write_endpoint_locked!(self, &self.id, &self.index(), supported_command_classes)
        {
            *handle = ccs;
        }
    }

    pub fn can_sleep(&self) -> bool {
        !self.protocol_data.listening && self.protocol_data.frequent_listening.is_none()
    }

    /// Pings the node and returns whether it responded or not.
    pub async fn ping(&self) -> ControllerCommandResult<bool> {
        // ^ Although this is a node command, the only errors we want to surface are controller errors
        let cc = NoOperationCC {}.with_destination(self.id.into());
        let result = self.driver.exec_node_command(&cc, None).await;
        match result {
            Ok(_) => Ok(true),
            Err(ExecNodeCommandError::NodeNoAck) => Ok(false),
            Err(ExecNodeCommandError::Controller(e)) => Err(e),
            Err(ExecNodeCommandError::NodeTimeout) => panic!("NoOperation CC should not time out"),
        }
    }
}

impl Endpoint for Node<'_> {
    fn node_id(&self) -> NodeId {
        self.id
    }

    fn get_node(&self) -> &Node<'_> {
        // A node IS the root endpoint
        self
    }

    fn index(&self) -> EndpointIndex {
        EndpointIndex::Root
    }
}
