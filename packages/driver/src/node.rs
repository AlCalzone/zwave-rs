use crate::{Controller, ControllerCommandResult, Driver, ExecNodeCommandError, Ready};
use cache::EndpointValueCache;
use zwave_cc::commandclass::{CCAddressable, NoOperationCC};
use zwave_core::{definitions::*, submodule};
use zwave_logging::loggers::node::NodeLogger;

submodule!(interview);
submodule!(storage);
submodule!(cc_api);
mod cache;

// macro_rules! read {
//     ($self:ident, $node_id:expr, $field:ident) => {
//         $self
//             .driver
//             .get_node_storage($node_id)
//             .map(|storage| storage.$field)
//     };
// }

macro_rules! read_locked {
    ($self:ident, $node_id:expr, $field:ident; deref) => {
        $self
            .controller
            .node_storage()
            .get($node_id)
            .map(|storage| *storage.$field)
    };
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .controller
            .node_storage()
            .get($node_id)
            .map(|storage| storage.$field)
    };
}

macro_rules! write_locked {
    ($self:ident, $node_id:expr, $field:ident) => {
        $self
            .controller
            .node_storage_mut()
            .get_mut($node_id)
            .map(|storage| &mut storage.$field)
    };
}

macro_rules! read_endpoint_locked {
    // ($self:ident, $node_id:expr, $endpoint_index:expr, $field:ident; deref) => {
    //     $self
    //         .driver
    //         .storage
    //         .endpoints()
    //         .get(&(*$node_id, *$endpoint_index))
    //         .map(|storage| *storage.$field)
    // };
    ($self:ident, $node_id:expr, $endpoint_index:expr, $field:ident) => {
        $self
            .controller
            .endpoint_storage()
            .get(&(*$node_id, *$endpoint_index))
            .map(|storage| &storage.$field)
    };
}

macro_rules! write_endpoint_locked {
    ($self:ident, $node_id:expr, $endpoint_index:expr, $field:ident) => {
        $self
            .controller
            .endpoint_storage_mut()
            .get_mut(&(*$node_id, *$endpoint_index))
            .map(|storage| &mut storage.$field)
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
    controller: &'a Controller<'a, Ready>,
}

pub trait EndpointLike<'a> {
    fn node_id(&self) -> NodeId;
    fn get_node(&'a self) -> &'a Node<'a>;
    fn index(&self) -> EndpointIndex;
    fn value_cache(&'a self) -> EndpointValueCache<'a>;

    fn modify_cc_info(&self, cc: CommandClasses, info: &PartialCommandClassInfo);
    fn remove_cc(&self, cc: CommandClasses);

    fn supported_command_classes(&self) -> Vec<CommandClasses>;
    fn controlled_command_classes(&self) -> Vec<CommandClasses>;
    fn supports_cc(&self, cc: CommandClasses) -> bool;
    fn controls_cc(&self, cc: CommandClasses) -> bool;
    fn get_cc_version(&self, cc: CommandClasses) -> Option<u8>;

    fn logger(&self) -> NodeLogger;

    // TODO: Add the rest
}

impl<'a> Node<'a> {
    pub fn new(
        id: NodeId,
        protocol_data: NodeInformationProtocolData,
        controller: &'a Controller<Ready>,
    ) -> Self {
        Self {
            id,
            protocol_data,
            controller,
        }
    }

    pub(crate) fn controller(&self) -> &Controller<Ready> {
        self.controller
    }

    pub(crate) fn driver(&self) -> &Driver {
        self.controller.driver()
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn get_endpoint(&self, index: u8) -> Endpoint {
        Endpoint::new(self, index, self.controller)
    }

    pub fn interview_stage(&self) -> InterviewStage {
        read_locked!(self, &self.id, interview_stage).unwrap_or(InterviewStage::None)
    }

    pub fn set_interview_stage(&self, interview_stage: InterviewStage) {
        if let Some(handle) = write_locked!(self, &self.id, interview_stage) {
            *handle = interview_stage;
        }
    }

    pub fn protocol_data(&self) -> &NodeInformationProtocolData {
        &self.protocol_data
    }

    pub fn can_sleep(&self) -> bool {
        !self.protocol_data.listening && self.protocol_data.frequent_listening.is_none()
    }

    /// Pings the node and returns whether it responded or not.
    pub async fn ping(&self) -> ControllerCommandResult<bool> {
        // ^ Although this is a node command, the only errors we want to surface are controller errors
        let cc = NoOperationCC {}.with_destination(self.id.into());
        let result = self.driver().exec_node_command(&cc.into(), None).await;
        match result {
            Ok(_) => Ok(true),
            Err(ExecNodeCommandError::NodeNoAck) => Ok(false),
            Err(ExecNodeCommandError::Controller(e)) => Err(e),
            Err(ExecNodeCommandError::NodeTimeout) => panic!("NoOperation CC should not time out"),
        }
    }
}

impl<'a> EndpointLike<'a> for Node<'a> {
    fn node_id(&self) -> NodeId {
        self.id
    }

    fn get_node(&self) -> &Node<'a> {
        // A node IS the root endpoint
        self
    }

    fn index(&self) -> EndpointIndex {
        EndpointIndex::Root
    }

    fn value_cache(&'a self) -> EndpointValueCache<'a> {
        EndpointValueCache::new(self, self.driver().value_cache())
    }

    fn modify_cc_info(&self, cc: CommandClasses, info: &PartialCommandClassInfo) {
        if let Some(cc_info) = write_endpoint_locked!(self, &self.id, &self.index(), cc_info) {
            cc_info
                .entry(cc)
                .and_modify(|cc_info| cc_info.merge(info))
                .or_insert_with(|| info.into());
        }
    }

    fn remove_cc(&self, cc: CommandClasses) {
        if let Some(cc_info) = write_endpoint_locked!(self, &self.id, &self.index(), cc_info) {
            cc_info.remove(&cc);
        }
    }

    fn supported_command_classes(&self) -> Vec<CommandClasses> {
        read_endpoint_locked!(self, &self.id, &self.index(), cc_info)
            .map(|map| {
                map.iter()
                    .filter_map(|(cc, info)| if info.supported() { Some(*cc) } else { None })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn controlled_command_classes(&self) -> Vec<CommandClasses> {
        read_endpoint_locked!(self, &self.id, &self.index(), cc_info)
            .map(|map| {
                map.iter()
                    .filter_map(|(cc, info)| if info.controlled() { Some(*cc) } else { None })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn supports_cc(&self, cc: CommandClasses) -> bool {
        read_endpoint_locked!(self, &self.id, &self.index(), cc_info)
            .map(|map| map.get(&cc).map(|cc| cc.supported()))
            .flatten()
            .unwrap_or(false)
    }

    fn controls_cc(&self, cc: CommandClasses) -> bool {
        read_endpoint_locked!(self, &self.id, &self.index(), cc_info)
            .map(|map| map.get(&cc).map(|cc| cc.controlled()))
            .flatten()
            .unwrap_or(false)
    }

    fn get_cc_version(&self, cc: CommandClasses) -> Option<u8> {
        read_endpoint_locked!(self, &self.id, &self.index(), cc_info)
            .map(|map| map.get(&cc).map(|cc| cc.version()))
            .flatten()
    }

    fn logger(&self) -> NodeLogger {
        self.controller
            .driver()
            .node_log(self.node_id(), self.index())
    }
}

pub struct Endpoint<'a> {
    node: &'a Node<'a>,
    index: u8,
    controller: &'a Controller<'a, Ready>,
}

impl<'a> Endpoint<'a> {
    pub fn new(node: &'a Node<'a>, index: u8, controller: &'a Controller<Ready>) -> Self {
        Self {
            node,
            index,
            controller,
        }
    }

    pub fn controller(&self) -> &Controller<Ready> {
        self.controller
    }
}

impl<'a> EndpointLike<'a> for Endpoint<'a> {
    fn node_id(&self) -> NodeId {
        self.node.id()
    }

    fn get_node(&'a self) -> &Node<'a> {
        self.node
    }

    fn index(&self) -> EndpointIndex {
        EndpointIndex::Endpoint(self.index)
    }

    fn value_cache(&'a self) -> EndpointValueCache<'a> {
        EndpointValueCache::new(self, self.get_node().driver().value_cache())
    }

    fn modify_cc_info(&self, cc: CommandClasses, info: &PartialCommandClassInfo) {
        if let Some(cc_info) = write_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
        {
            cc_info
                .entry(cc)
                .and_modify(|cc_info| cc_info.merge(info))
                .or_insert_with(|| info.into());
        }
    }

    fn remove_cc(&self, cc: CommandClasses) {
        if let Some(cc_info) = write_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
        {
            cc_info.remove(&cc);
        }
    }

    fn supported_command_classes(&self) -> Vec<CommandClasses> {
        read_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
            .map(|map| {
                map.iter()
                    .filter_map(|(cc, info)| if info.supported() { Some(*cc) } else { None })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn controlled_command_classes(&self) -> Vec<CommandClasses> {
        read_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
            .map(|map| {
                map.iter()
                    .filter_map(|(cc, info)| if info.controlled() { Some(*cc) } else { None })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn supports_cc(&self, cc: CommandClasses) -> bool {
        read_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
            .map(|map| map.get(&cc).map(|cc| cc.supported()))
            .flatten()
            .unwrap_or(false)
    }

    fn controls_cc(&self, cc: CommandClasses) -> bool {
        read_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
            .map(|map| map.get(&cc).map(|cc| cc.controlled()))
            .flatten()
            .unwrap_or(false)
    }

    fn get_cc_version(&self, cc: CommandClasses) -> Option<u8> {
        read_endpoint_locked!(self, &self.node_id(), &self.index(), cc_info)
            .map(|map| map.get(&cc).map(|cc| cc.version()))
            .flatten()
    }

    fn logger(&self) -> NodeLogger {
        self.controller
            .driver()
            .node_log(self.node_id(), self.index())
    }
}
