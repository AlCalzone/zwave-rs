use crate::driver_api::DriverApi;
use crate::error::Error;
use crate::ControllerCommandError;
use crate::DriverState;
use std::time::Duration;
use thiserror::Error;
use typed_builder::TypedBuilder;
use zwave_cc::commandclass::IntoCCSequence;
use zwave_cc::commandclass::WithAddress;
use zwave_cc::prelude::*;
use zwave_core::prelude::*;
use zwave_serial::command::SendDataRequest;
use zwave_serial::prelude::*;

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

impl DriverApi {
    // FIXME: Assert that the driver is in the Ready state

    pub async fn exec_node_command(
        &self,
        cc: &WithAddress<CC>,
        _options: Option<&ExecNodeCommandOptions>,
    ) -> ExecNodeCommandResult<Option<CC>> {
        // Create a CC sequence in order to be able to handle CCs that require sequencing
        let mut sequence = cc.clone().into_cc_sequence();

        // For each CC in the sequence, send the CC and handle the reponse if needed
        loop {
            let Some(mut cc) = sequence.next(self.own_node_id()) else {
                // We should not end up here, but if we do, return nothing
                return Ok(None);
            };

            // TODO: Maybe there's a better way to set the auth and enc key, but I don't know what it is
            #[allow(clippy::single_match)]
            match cc.as_mut() {
                CC::SecurityCCCommandEncapsulation(security_cc) => {
                    if let Some(sec_man) = self.security_manager() {
                        security_cc.set_auth_key(sec_man.auth_key().to_vec());
                        security_cc.set_enc_key(sec_man.enc_key().to_vec());
                    }
                }
                _ => {}
            }

            let partial_result = self.exec_node_command_internal(&cc, None).await?;

            if sequence.is_finished() {
                return Ok(partial_result);
            }

            if let Some(cc) = &partial_result {
                sequence.handle_response(cc);
            }
        }
    }

    async fn exec_node_command_internal(
        &self,
        cc: &WithAddress<CC>,
        _options: Option<&ExecNodeCommandOptions>,
    ) -> ExecNodeCommandResult<Option<CC>> {
        // FIXME: In some cases, the nodes' responses are received BEFORE
        // the controller callback is received. We don't handle this case yet.
        let node_id = match cc.address().destination {
            Destination::Singlecast(node_id) => node_id,
            Destination::Multicast(_) => todo!("Multicast not implemented yet"),
            Destination::Broadcast => NodeId::broadcast(),
        };

        let controller_command = SendDataRequest::builder()
            .node_id(node_id)
            .command(cc.clone().into())
            .build();

        let controller_command_result = self
            .exec_controller_command(controller_command, None)
            .await
            .map_err(ControllerCommandError::from)?
            .expect("SendData should always be answered by the controller");

        match controller_command_result {
            Command::SendDataResponse(resp) => {
                if !resp.is_ok() {
                    todo!("Handle failed SendData response")
                }
            }
            Command::SendDataCallback(cb) => {
                if !cb.is_ok() {
                    // FIXME: Use callback information in statistics
                    // FIXME: This is not necessarily NoAck, it could be Fail too
                    return Err(ExecNodeCommandError::NodeNoAck);
                }
            }
            other => {
                panic!("Unexpected command response {:?} to SendDataRequest", other);
            }
        }

        if !cc.expects_response() {
            return Ok(None);
        }

        // TODO: Consider registering this earlier (after the SendDataRequest is sent)
        let awaited_cc_response = {
            let cc = cc.clone();
            self.await_cc(
                Box::new(move |recv| test_cc_response(&cc, recv)),
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|_| {
                ControllerCommandError::Unexpected(
                    "Unexpected internal error while registering CC response awaiter".to_string(),
                )
            })?
        };

        match awaited_cc_response.try_await().await {
            Ok(recv) => Ok(Some(recv.unwrap())),
            Err(Error::Timeout) => Err(ExecNodeCommandError::NodeTimeout),
            Err(_) => {
                panic!("Unexpected internal error while waiting for CC response");
            }
        }
    }
}
