use crate::{
    Controller, ControllerCommandResult, Driver, EndpointStateRef, ExecNodeCommandError,
    NodeStateRef, Ready,
};
use cache::EndpointValueCache;
use zwave_cc::commandclass::{AsDestination, CCAddressable, Destination, NoOperationCC};
use zwave_core::{definitions::*, submodule};
use zwave_logging::loggers::node::NodeLogger;

submodule!(interview);
submodule!(storage);
submodule!(cc_api);
mod cache;

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

    fn logger(&self) -> NodeLogger<'_>;

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

    pub(crate) fn controller(&self) -> &'_ Controller<'_, Ready> {
        self.controller
    }

    pub(crate) fn driver(&self) -> &Driver {
        self.controller.driver()
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn endpoint(&self, index: u8) -> Endpoint<'_> {
        Endpoint::new(self, index, self.controller)
    }

    pub fn interview_stage(&self) -> InterviewStage {
        self.state()
            .interview_stage()
            .unwrap_or(InterviewStage::None)
    }

    pub fn set_interview_stage(&self, interview_stage: InterviewStage) {
        self.state().set_interview_stage(interview_stage);
    }

    pub fn protocol_data(&self) -> &NodeInformationProtocolData {
        &self.protocol_data
    }

    pub fn can_sleep(&self) -> bool {
        !self.protocol_data.listening && self.protocol_data.frequent_listening.is_none()
    }

    pub fn has_security_class(&self, security_class: SecurityClass) -> Option<bool> {
        self.state().has_security_class(security_class)
    }

    pub fn set_security_class(&self, security_class: SecurityClass, granted: bool) {
        self.state().set_security_class(security_class, granted);
    }

    pub fn highest_security_class(&self) -> Option<SecurityClass> {
        self.state().highest_security_class()
    }

    fn state(&self) -> NodeStateRef<'_> {
        self.controller.node_state(self.id)
    }

    fn endpoint_state(&self) -> EndpointStateRef<'_> {
        self.state().endpoint(EndpointIndex::Root)
    }

    /// Pings the node and returns whether it responded or not.
    pub async fn ping(&self) -> ControllerCommandResult<bool> {
        // ^ Although this is a node command, the only errors we want to surface are controller errors
        let cc = NoOperationCC {}.with_destination(self.as_destination());
        let result = self.driver().exec_node_command(&cc.into(), None).await;
        match result {
            Ok(_) => Ok(true),
            Err(ExecNodeCommandError::NodeNoAck) => Ok(false),
            Err(ExecNodeCommandError::Controller(e)) => Err(e),
            Err(ExecNodeCommandError::NodeTimeout) => panic!("NoOperation CC should not time out"),
        }
    }
}

impl AsDestination for Node<'_> {
    fn as_destination(&self) -> Destination {
        self.id.into()
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
        self.endpoint_state().merge_command_class_info(cc, info);
    }

    fn remove_cc(&self, cc: CommandClasses) {
        self.endpoint_state().remove_command_class(cc);
    }

    fn supported_command_classes(&self) -> Vec<CommandClasses> {
        self.endpoint_state().supported_command_classes()
    }

    fn controlled_command_classes(&self) -> Vec<CommandClasses> {
        self.endpoint_state().controlled_command_classes()
    }

    fn supports_cc(&self, cc: CommandClasses) -> bool {
        self.endpoint_state().supports_command_class(cc)
    }

    fn controls_cc(&self, cc: CommandClasses) -> bool {
        self.endpoint_state().controls_command_class(cc)
    }

    fn get_cc_version(&self, cc: CommandClasses) -> Option<u8> {
        self.endpoint_state().command_class_version(cc)
    }

    fn logger(&self) -> NodeLogger<'_> {
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

    pub fn controller(&self) -> &'_ Controller<'_, Ready> {
        self.controller
    }

    fn state(&self) -> EndpointStateRef<'_> {
        self.controller
            .node_state(self.node_id())
            .endpoint(self.index())
    }
}

impl<'a> EndpointLike<'a> for Endpoint<'a> {
    fn node_id(&self) -> NodeId {
        self.node.id()
    }

    fn get_node(&'a self) -> &'a Node<'a> {
        self.node
    }

    fn index(&self) -> EndpointIndex {
        EndpointIndex::Endpoint(self.index)
    }

    fn value_cache(&'a self) -> EndpointValueCache<'a> {
        EndpointValueCache::new(self, self.get_node().driver().value_cache())
    }

    fn modify_cc_info(&self, cc: CommandClasses, info: &PartialCommandClassInfo) {
        self.state().merge_command_class_info(cc, info);
    }

    fn remove_cc(&self, cc: CommandClasses) {
        self.state().remove_command_class(cc);
    }

    fn supported_command_classes(&self) -> Vec<CommandClasses> {
        self.state().supported_command_classes()
    }

    fn controlled_command_classes(&self) -> Vec<CommandClasses> {
        self.state().controlled_command_classes()
    }

    fn supports_cc(&self, cc: CommandClasses) -> bool {
        self.state().supports_command_class(cc)
    }

    fn controls_cc(&self, cc: CommandClasses) -> bool {
        self.state().controls_command_class(cc)
    }

    fn get_cc_version(&self, cc: CommandClasses) -> Option<u8> {
        self.state().command_class_version(cc)
    }

    fn logger(&self) -> NodeLogger<'_> {
        self.controller
            .driver()
            .node_log(self.node_id(), self.index())
    }
}
