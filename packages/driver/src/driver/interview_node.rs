use crate::{error::*, Driver, Ready};
use zwave_core::definitions::*;

impl Driver<Ready> {
    pub async fn interview_node(&self, _node_id: &NodeId) -> Result<()> {
        // // FIXME: Don't hold the lock for longer than necessary
        // let mut nodes = self.nodes_mut();
        // let node = nodes.get_mut(node_id).unwrap();

        // if node.interview_stage() == &InterviewStage::None {
        //     node.set_interview_stage(InterviewStage::ProtocolInfo);

        //     let protocol_info = self.get_node_protocol_info(node_id, None).await?;
        //     println!("Node {:?} protocol info: {:?}", node_id, protocol_info);
        // }

        Ok(())
    }
}
