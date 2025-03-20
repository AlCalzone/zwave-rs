use std::time::Duration;

use super::ExecControllerCommandError;
use super::{ControllerCommandError, Driver};
use crate::error::Error;
use thiserror::Error;
use typed_builder::TypedBuilder;
use zwave_cc::commandclass::IntoCCSequence;
use zwave_cc::commandclass::WithAddress;
use zwave_cc::prelude::*;
use zwave_core::prelude::*;
use zwave_serial::command::SendDataRequest;
use zwave_serial::prelude::*;

impl Driver {
    pub async fn exec_node_command(
        &self,
        cc: &WithAddress<CC>,
        _options: Option<&ExecNodeCommandOptions>,
    ) -> ExecNodeCommandResult<Option<CC>> {
        // Create a CC sequence in order to be able to handle CCs that require sequencing
        let mut sequence = cc.clone().into_cc_sequence();

        let node_id = match cc.address().destination {
            Destination::Singlecast(node_id) => node_id,
            Destination::Multicast(_) => todo!("Multicast not implemented yet"),
            Destination::Broadcast => NodeId::broadcast(),
        };

        // For each CC in the sequence, send the CC and handle the reponse if needed
        loop {
            let ctx = self.get_cc_encoding_context(node_id);
            let Some(cc) = sequence.next(&ctx) else {
                // We should not end up here, but if we do, return nothing
                return Ok(None);
            };

            let partial_result = self.exec_node_command_internal(node_id, &cc, None).await?;

            if sequence.is_finished() {
                return Ok(partial_result);
            }

            if let Some(cc) = &partial_result {
                sequence.handle_response(cc);
            }
        }
    }

    fn get_cc_encoding_context(&self, destination_node_id: NodeId) -> CCEncodingContext {
        CCEncodingContext::builder()
            .own_node_id(self.serial_api.storage.own_node_id())
            .node_id(destination_node_id)
            .security_manager(self.storage.security_manager().clone())
            .build()
    }

    async fn exec_node_command_internal(
        &self,
        node_id: NodeId,
        cc: &CC,
        _options: Option<&ExecNodeCommandOptions>,
    ) -> ExecNodeCommandResult<Option<CC>> {
        // FIXME: In some cases, the nodes' responses are received BEFORE
        // the controller callback is received. We don't handle this case yet.

        let ctx = self.get_cc_encoding_context(node_id);
        let serialized = cc.clone().as_raw(&ctx);

        let controller_command = SendDataRequest::builder()
            .node_id(node_id)
            .command(serialized.into())
            .build();

        let controller_command_result =
            self.exec_controller_command(controller_command, None).await;

        match controller_command_result {
            Ok(Some(Command::SendDataResponse(_))) | Ok(Some(Command::SendDataCallback(_))) => {
                // All good, this is expected
            }
            Err(ExecControllerCommandError::ResponseNOK(Command::SendDataResponse(resp))) => {
                todo!("Handle failed SendData response")
            }
            Err(ExecControllerCommandError::CallbackNOK(Command::SendDataCallback(cb))) => {
                // FIXME: Use callback information in statistics
                // FIXME: This is not necessarily NoAck, it could be Fail too
                return Err(ExecNodeCommandError::NodeNoAck);
            }
            other => {
                panic!("Unexpected command response {:?} to SendDataRequest", other);
            }
        }

        if !cc.expects_response() {
            return Ok(None);
        }

        let awaited_cc_response = {
            let cc = cc.clone().with_destination(node_id.into());
            self.await_cc(
                Box::new(move |recv| test_cc_response(&cc, recv)),
                Some(Duration::from_secs(10)),
            )
            .await
        };

        match awaited_cc_response {
            Ok(recv) => Ok(Some(recv.unwrap())),
            Err(Error::Timeout) => Err(ExecNodeCommandError::NodeTimeout),
            Err(_) => {
                panic!("Unexpected internal error while waiting for CC response");
            }
        }
    }
}

#[derive(TypedBuilder, Default, Clone)]
pub struct ExecNodeCommandOptions {}

/// The result of a node command execution
pub type ExecNodeCommandResult<T> = Result<T, ExecNodeCommandError>;

#[derive(Error, Debug)]
/// Defines the possible errors for a node command execution
pub enum ExecNodeCommandError {
    #[error("Controller command error: {0}")]
    Controller(#[from] ControllerCommandError),
    #[error("The node did not acknowledge the command")]
    NodeNoAck,
    #[error("Timed out waiting for a response from the node")]
    NodeTimeout,
}

/// Tests if the given CC response is the expected CC response to the given CC request
fn test_cc_response<C>(request: &WithAddress<C>, response: &WithAddress<CC>) -> bool
where
    C: Into<CC> + CCBase + CCId,
{
    if !request.expects_response() {
        return false;
    }

    if let Destination::Singlecast(target) = request.address().destination {
        response.address().source_node_id == target
        // FIXME: Consider encapsulation
            && request.cc_id() == response.cc_id()
            && request.test_response(response)
    } else {
        false
    }
}
