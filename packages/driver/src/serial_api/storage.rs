use zwave_core::prelude::*;
use zwave_core::util::Locked;

/// Storage shared between the Serial API and driver actors, containing information
/// that is needed to correctly parse and serialize commands.
pub(crate) struct SerialApiStorage {
    home_id: Locked<Id32>,
    own_node_id: Locked<NodeId>,
    node_id_type: Locked<NodeIdType>,
    sdk_version: Locked<Option<Version>>,
}

impl SerialApiStorage {
    pub fn new(node_id_type: NodeIdType) -> Self {
        Self {
            home_id: Locked::new(Id32::default()),
            own_node_id: Locked::new(NodeId::unspecified()),
            node_id_type: Locked::new(node_id_type),
            sdk_version: Locked::new(None),
        }
    }

    pub(crate) fn home_id(&self) -> &Locked<Id32> {
        &self.home_id
    }

    pub(crate) fn own_node_id(&self) -> &Locked<NodeId> {
        &self.own_node_id
    }

    pub(crate) fn node_id_type(&self) -> &Locked<NodeIdType> {
        &self.node_id_type
    }

    pub(crate) fn sdk_version(&self) -> &Locked<Option<Version>> {
        &self.sdk_version
    }
}
