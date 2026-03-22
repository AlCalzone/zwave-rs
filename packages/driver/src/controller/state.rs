use zwave_pal::prelude::*;
use super::{Controller, Ready};
use crate::{EndpointStorage, InterviewStage};
use zwave_core::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct NodeStateRef<'a> {
    controller: &'a Controller<'a, Ready>,
    node_id: NodeId,
}

#[derive(Clone, Copy)]
pub(crate) struct EndpointStateRef<'a> {
    controller: &'a Controller<'a, Ready>,
    node_id: NodeId,
    endpoint_index: EndpointIndex,
}

impl<'a> Controller<'a, Ready> {
    pub(crate) fn node_state(&'a self, node_id: NodeId) -> NodeStateRef<'a> {
        NodeStateRef {
            controller: self,
            node_id,
        }
    }
}

impl<'a> NodeStateRef<'a> {
    pub(crate) fn endpoint(self, endpoint_index: EndpointIndex) -> EndpointStateRef<'a> {
        EndpointStateRef {
            controller: self.controller,
            node_id: self.node_id,
            endpoint_index: endpoint_index.to_canonical(),
        }
    }

    pub(crate) fn exists(self) -> bool {
        self.controller
            .state
            .nodes
            .inspect(|nodes| nodes.contains_key(&self.node_id))
    }

    pub(crate) fn protocol_data(self) -> Option<NodeInformationProtocolData> {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                // We clone the protocol data from storage to avoid lots of node methods
                // needing an Option as the return type in case the node was removed after
                // the call to get_node
                .map(|storage| storage.protocol_data.clone())
        })
    }

    pub(crate) fn interview_stage(self) -> Option<InterviewStage> {
        self.controller
            .state
            .nodes
            .inspect(|nodes| nodes.get(&self.node_id).map(|storage| storage.interview_stage))
    }

    pub(crate) fn set_interview_stage(self, interview_stage: InterviewStage) -> bool {
        self.controller.state.nodes.update(|nodes| {
            let Some(storage) = nodes.get_mut(&self.node_id) else {
                return false;
            };
            storage.interview_stage = interview_stage;
            true
        })
    }

    pub(crate) fn endpoint_exists(self, endpoint_index: EndpointIndex) -> bool {
        self.endpoint(endpoint_index).exists()
    }

    pub(crate) fn ensure_endpoint(self, endpoint_index: EndpointIndex) -> bool {
        self.endpoint(endpoint_index).ensure_exists()
    }
}

impl<'a> EndpointStateRef<'a> {
    pub(crate) fn exists(self) -> bool {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index))
                .is_some()
        })
    }

    pub(crate) fn ensure_exists(self) -> bool {
        self.controller.state.nodes.update(|nodes| {
            let Some(node) = nodes.get_mut(&self.node_id) else {
                return false;
            };

            node.endpoints
                .entry(self.endpoint_index)
                .or_insert_with(EndpointStorage::new);
            true
        })
    }

    pub(crate) fn supported_command_classes(self) -> Vec<CommandClasses> {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index))
                .map(|endpoint| {
                    endpoint
                        .cc_info
                        .iter()
                        .filter_map(|(cc, info)| if info.supported { Some(*cc) } else { None })
                        .collect()
                })
                .unwrap_or_default()
        })
    }

    pub(crate) fn controlled_command_classes(self) -> Vec<CommandClasses> {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index))
                .map(|endpoint| {
                    endpoint
                        .cc_info
                        .iter()
                        .filter_map(|(cc, info)| if info.controlled { Some(*cc) } else { None })
                        .collect()
                })
                .unwrap_or_default()
        })
    }

    pub(crate) fn remove_command_class(self, command_class: CommandClasses) -> bool {
        self.controller.state.nodes.update(|nodes| {
            let Some(node) = nodes.get_mut(&self.node_id) else {
                return false;
            };
            let Some(endpoint) = node.endpoints.get_mut(&self.endpoint_index) else {
                return false;
            };
            endpoint.cc_info.remove(&command_class);
            true
        })
    }

    pub(crate) fn supports_command_class(self, command_class: CommandClasses) -> bool {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index))
                .and_then(|endpoint| endpoint.cc_info.get(&command_class))
                .map(|info| info.supported)
                .unwrap_or(false)
        })
    }

    pub(crate) fn controls_command_class(self, command_class: CommandClasses) -> bool {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index))
                .and_then(|endpoint| endpoint.cc_info.get(&command_class))
                .map(|info| info.controlled)
                .unwrap_or(false)
        })
    }

    pub(crate) fn command_class_version(self, command_class: CommandClasses) -> Option<u8> {
        self.controller.state.nodes.inspect(|nodes| {
            nodes.get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index))
                .and_then(|endpoint| endpoint.cc_info.get(&command_class))
                .map(|info| info.version)
        })
    }

    pub(crate) fn merge_command_class_info(
        self,
        command_class: CommandClasses,
        info: &PartialCommandClassInfo,
    ) -> bool {
        self.controller.state.nodes.update(|nodes| {
            let Some(node) = nodes.get_mut(&self.node_id) else {
                return false;
            };
            let endpoint = node
                .endpoints
                .entry(self.endpoint_index)
                .or_insert_with(EndpointStorage::new);

            endpoint
                .cc_info
                .entry(command_class)
                .and_modify(|cc_info| cc_info.merge(info))
                .or_insert_with(|| info.into());
            true
        })
    }
}
