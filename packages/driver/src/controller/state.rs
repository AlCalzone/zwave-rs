use super::{Controller, Ready};
use crate::{EndpointStorage, InterviewStage, NodeStorage};
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
    fn inspect_node<R>(self, f: impl FnOnce(Option<&NodeStorage>) -> R) -> R {
        self.controller
            .state
            .nodes
            .inspect(|nodes| f(nodes.get(&self.node_id)))
    }

    fn update_node<R>(self, f: impl FnOnce(Option<&mut NodeStorage>) -> R) -> R {
        self.controller
            .state
            .nodes
            .update(|nodes| f(nodes.get_mut(&self.node_id)))
    }

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
        self.inspect_node(|storage| {
            let storage = storage?;
            // We clone the protocol data from storage to avoid lots of node methods
            // needing an Option as the return type in case the node was removed after
            // the call to get_node
            Some(storage.protocol_data.clone())
        })
    }

    pub(crate) fn interview_stage(self) -> Option<InterviewStage> {
        self.inspect_node(|storage| {
            let storage = storage?;
            Some(storage.interview_stage)
        })
    }

    pub(crate) fn set_interview_stage(self, interview_stage: InterviewStage) -> bool {
        self.update_node(|storage| {
            let Some(storage) = storage else {
                return false;
            };
            storage.interview_stage = interview_stage;
            true
        })
    }

    pub(crate) fn has_security_class(self, security_class: SecurityClass) -> Option<bool> {
        self.inspect_node(|storage| storage?.security_classes.get(&security_class).copied())
    }

    pub(crate) fn set_security_class(self, security_class: SecurityClass, granted: bool) -> bool {
        self.update_node(|storage| {
            let Some(storage) = storage else {
                return false;
            };
            storage.security_classes.insert(security_class, granted);
            true
        })
    }

    pub(crate) fn highest_security_class(self) -> Option<SecurityClass> {
        self.inspect_node(|storage| {
            storage?
                .security_classes
                .iter()
                .filter_map(|(security_class, granted)| granted.then_some(*security_class))
                .max()
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
    fn inspect_endpoint<R>(self, f: impl FnOnce(Option<&EndpointStorage>) -> R) -> R {
        self.controller.state.nodes.inspect(|nodes| {
            f(nodes
                .get(&self.node_id)
                .and_then(|node| node.endpoints.get(&self.endpoint_index)))
        })
    }

    fn update_node<R>(self, f: impl FnOnce(Option<&mut NodeStorage>) -> R) -> R {
        self.controller
            .state
            .nodes
            .update(|nodes| f(nodes.get_mut(&self.node_id)))
    }

    fn update_endpoint<R>(self, f: impl FnOnce(Option<&mut EndpointStorage>) -> R) -> R {
        self.update_node(|node| {
            f(node.and_then(|node| node.endpoints.get_mut(&self.endpoint_index)))
        })
    }

    pub(crate) fn exists(self) -> bool {
        self.inspect_endpoint(|endpoint| endpoint.is_some())
    }

    pub(crate) fn ensure_exists(self) -> bool {
        self.update_node(|node| {
            let Some(node) = node else {
                return false;
            };

            node.endpoints
                .entry(self.endpoint_index)
                .or_insert_with(EndpointStorage::new);
            true
        })
    }

    pub(crate) fn supported_command_classes(self) -> Vec<CommandClasses> {
        self.inspect_endpoint(|endpoint| {
            endpoint
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
        self.inspect_endpoint(|endpoint| {
            endpoint
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
        self.update_endpoint(|endpoint| {
            let Some(endpoint) = endpoint else {
                return false;
            };
            endpoint.cc_info.remove(&command_class);
            true
        })
    }

    pub(crate) fn supports_command_class(self, command_class: CommandClasses) -> bool {
        self.inspect_endpoint(|endpoint| {
            endpoint
                .and_then(|endpoint| endpoint.cc_info.get(&command_class))
                .map(|info| info.supported)
                .unwrap_or(false)
        })
    }

    pub(crate) fn controls_command_class(self, command_class: CommandClasses) -> bool {
        self.inspect_endpoint(|endpoint| {
            endpoint
                .and_then(|endpoint| endpoint.cc_info.get(&command_class))
                .map(|info| info.controlled)
                .unwrap_or(false)
        })
    }

    pub(crate) fn command_class_version(self, command_class: CommandClasses) -> Option<u8> {
        self.inspect_endpoint(|endpoint| {
            endpoint
                .and_then(|endpoint| endpoint.cc_info.get(&command_class))
                .map(|info| info.version)
        })
    }

    pub(crate) fn merge_command_class_info(
        self,
        command_class: CommandClasses,
        info: &PartialCommandClassInfo,
    ) -> bool {
        self.update_node(|node| {
            let Some(node) = node else {
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
