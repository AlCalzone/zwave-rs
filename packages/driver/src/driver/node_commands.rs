use std::time::Duration;

use crate::error::Error;
use crate::exec_background_task;
use crate::ControllerCommandError;
use crate::Driver;
use crate::Ready;
use thiserror::Error;
use typed_builder::TypedBuilder;
use zwave_cc::commandclass::CCRequest;
use zwave_cc::commandclass::CC;
use zwave_core::prelude::*;
use zwave_serial::command::Command;
use zwave_serial::command::CommandBase;
use zwave_serial::command::SendDataRequest;

#[derive(TypedBuilder, Default, Clone)]
pub struct ExecNodeCommandOptions {}

/// The node command execution.
pub type ExecNodeCommandResult<T> = Result<T, ExecNodeCommandError>;

#[derive(Error, Debug)]
/// Defines the possible errors for a node command execution
pub enum ExecNodeCommandError {
    #[error("Controller command error: {0}")]
    Controller(#[from] ControllerCommandError),
    #[error("Timed out waiting for a response from the node")]
    NodeTimeout,
}

impl Driver<Ready> {
    pub async fn exec_node_command<C>(
        &mut self,
        node_id: NodeId, // FIXME: Use the Destination enum and handle Multicast/Broadcast
        cc: C,
        options: Option<&ExecNodeCommandOptions>,
    ) -> ExecNodeCommandResult<Option<CC>>
    where
        C: CCRequest + Clone + 'static,
        CC: From<C>,
    {
        // FIXME: In some cases, the nodes' responses are received BEFORE
        // the controller callback is received. We don't handle this case yet.

        let controller_command = SendDataRequest::builder()
            .node_id(node_id)
            .command(cc.clone().into())
            .build();

        let controller_command_result = self
            .exec_controller_command(controller_command, None)
            .await
            .map_err(|e| ControllerCommandError::from(e))?
            .expect("SendData should always be answered by the controller");

        match controller_command_result {
            Command::SendDataResponse(resp) => {
                if !resp.is_ok() {
                    todo!("Handle failed SendData response")
                }
            }
            Command::SendDataCallback(cb) => {
                if !cb.is_ok() {
                    todo!("Handle failed SendData callback")
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
                Box::new(move |recv| cc.test_response(recv)),
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
            Ok(recv) => Ok(Some(recv)),
            Err(Error::Timeout) => Err(ExecNodeCommandError::NodeTimeout),
            Err(_) => {
                panic!("Unexpected internal error while waiting for CC response");
            }
        }
    }
}
