use super::{
    awaited::{AwaitedRef, Predicate},
    serial_api_machine::{
        SerialApiMachine, SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineResult,
        SerialApiMachineState,
    },
    storage::{self, DriverStorage},
    Driver, MainTaskCommandSender, SerialTaskCommandSender,
};
use crate::error::{Error, Result};
use crate::{dispatch_async, DriverState};
use core::panic;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use zwave_cc::commandclass::{CCEncodingContext, WithAddress, CC};
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::{prelude::*, security::SecurityManager};
use zwave_logging::{
    loggers::{controller::ControllerLogger, driver::DriverLogger, node::NodeLogger},
    Direction,
};
use zwave_serial::{
    command::AsCommandRaw,
    frame::{ControlFlow, RawSerialFrame, SerialFrame},
    prelude::*,
};

#[derive(Clone)]
pub struct DriverApi {
    main_task_cmd: MainTaskCommandSender,
    serial_task_cmd: SerialTaskCommandSender,

    pub(crate) storage: Arc<DriverStorage>,
}

impl<S> Driver<S>
where
    S: DriverState,
{
    pub fn api(&self) -> DriverApi {
        DriverApi {
            serial_task_cmd: self.tasks.serial_cmd.clone(),
            main_task_cmd: self.tasks.main_cmd.clone(),
            storage: self.storage.clone(),
        }
    }
}

impl DriverApi {
    pub(crate) fn new(
        main_task_cmd: MainTaskCommandSender,
        serial_task_cmd: SerialTaskCommandSender,
        storage: Arc<DriverStorage>,
    ) -> Self {
        Self {
            main_task_cmd,
            serial_task_cmd,
            storage,
        }
    }

    /// Whether the given function type is supported
    pub fn supports_function(&self, function_type: FunctionType) -> bool {
        self.storage
            .controller()
            .as_ref()
            .map(|c| c.supported_function_types.contains(&function_type))
            .unwrap_or(false)
    }

    /// Write a frame to the serial port, returning a reference to the awaited ACK frame
    pub async fn write_serial(&self, frame: RawSerialFrame) -> Result<AwaitedRef<ControlFlow>> {
        // Register an awaiter for the ACK frame
        let ret = self
            .await_control_flow_frame(Box::new(|_| true), Some(Duration::from_millis(1600)))
            .await;
        // ...then send the frame
        let send_frame_result =
            dispatch_async!(&self.serial_task_cmd, SerialTaskCommand::SendFrame, frame)
                .expect("SerialTaskCommand::SendFrame failed");
        // Ensure it worked
        send_frame_result?;

        // Then return the awaiter
        ret
    }

    pub async fn await_control_flow_frame(
        &self,
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>,
    ) -> Result<AwaitedRef<ControlFlow>> {
        dispatch_async!(
            self.main_task_cmd,
            MainTaskCommand::RegisterAwaitedControlFlowFrame,
            predicate,
            timeout
        )
    }

    pub async fn await_command(
        &self,
        predicate: Predicate<Command>,
        timeout: Option<Duration>,
    ) -> Result<AwaitedRef<Command>> {
        dispatch_async!(
            self.main_task_cmd,
            MainTaskCommand::RegisterAwaitedCommand,
            predicate,
            timeout
        )
    }

    pub async fn await_cc(
        &self,
        predicate: Predicate<WithAddress<CC>>,
        timeout: Option<Duration>,
    ) -> Result<AwaitedRef<WithAddress<CC>>> {
        dispatch_async!(
            self.main_task_cmd,
            MainTaskCommand::RegisterAwaitedCC,
            predicate,
            timeout
        )
    }

    pub async fn get_next_callback_id(&self) -> Result<u8> {
        dispatch_async!(self.main_task_cmd, MainTaskCommand::GetNextCallbackId)
    }

    pub async fn execute_serial_api_command<C>(
        &self,
        mut command: C,
    ) -> Result<SerialApiMachineResult>
    where
        C: CommandRequest + AsCommandRaw + Into<Command> + Clone + 'static,
    {
        // Set up state machine and interpreter
        let mut state_machine = SerialApiMachine::new();

        // Give the command a callback ID if it needs one
        if command.needs_callback_id() && command.callback_id().is_none() {
            command.set_callback_id(Some(self.get_next_callback_id().await?));
        }

        let expects_response = command.expects_response();
        let expects_callback = command.expects_callback();
        let evaluate_condition =
            Box::new(
                move |condition: SerialApiMachineCondition| match condition {
                    SerialApiMachineCondition::ExpectsResponse => expects_response,
                    SerialApiMachineCondition::ExpectsCallback => expects_callback,
                },
            );

        // Handle all transitions/events from the state machine
        let mut next_input: Option<SerialApiMachineInput> = Some(SerialApiMachineInput::Start);

        // With multiple tasks involved, setting up the awaiters is very timing-sensitive and
        // prone to race conditions when set up just in time. Unless something is going horribly wrong,
        // setting up all awaiters before sending the command should be safe.
        let mut awaited_response: Option<AwaitedRef<Command>> = {
            let command = command.clone();
            Some(
                self.await_command(
                    Box::new(move |cmd| command.test_response(cmd)),
                    Some(Duration::from_millis(10000)),
                )
                .await
                .expect("await_command (response) failed"),
            )
        };
        let mut awaited_callback: Option<AwaitedRef<Command>> = {
            let command = command.clone();
            Some(
                self.await_command(
                    Box::new(move |cmd| command.test_callback(cmd)),
                    Some(Duration::from_millis(30000)),
                )
                .await
                .expect("await_command (callback) failed"),
            )
        };
        // The ACK awaiter is returned by the call to `write_serial()`
        let mut awaited_ack: Option<AwaitedRef<ControlFlow>> = None;

        while !state_machine.done() {
            if let Some(input) = next_input.take() {
                if let Some(transition) = state_machine.next(input, &evaluate_condition) {
                    let new_state = transition.new_state();

                    // Transition to the new state
                    state_machine.transition(new_state);

                    // Now check what needs to be done in the new state
                    match state_machine.state() {
                        SerialApiMachineState::Initial => (),
                        SerialApiMachineState::Sending => {
                            let ctx = CommandEncodingContext::builder()
                                .own_node_id(self.own_node_id())
                                .node_id_type(self.storage.node_id_type())
                                .sdk_version(self.storage.sdk_version())
                                .build();

                            let raw = command.as_raw(&ctx);
                            let frame = SerialFrame::Command(raw);
                            // Send the command to the controller
                            awaited_ack = Some(
                                self.write_serial(frame.into())
                                    .await
                                    .expect("write_serial failed"),
                            );
                            // and notify the state machine
                            next_input = Some(SerialApiMachineInput::FrameSent);

                            // And log the command information if this was a command
                            let node_id = match Into::<Command>::into(command.clone()) {
                                // FIXME: Extract the endpoint index aswell
                                Command::SendDataRequest(cmd) => Some(cmd.node_id),
                                _ => None,
                            };

                            if let Some(node_id) = node_id {
                                self.node_log(node_id, EndpointIndex::Root)
                                    .command(&command, Direction::Outbound);
                            } else {
                                self.controller_log().command(&command, Direction::Outbound);
                            }
                        }
                        SerialApiMachineState::WaitingForACK => {
                            // Wait for ACK, but also accept CAN and NAK
                            match awaited_ack
                                .take()
                                .expect("ACK awaiter already consumed")
                                .try_await()
                                .await
                            {
                                Ok(frame) => {
                                    next_input = Some(match frame {
                                        ControlFlow::ACK => SerialApiMachineInput::ACK,
                                        ControlFlow::NAK => SerialApiMachineInput::NAK,
                                        ControlFlow::CAN => SerialApiMachineInput::CAN,
                                    });
                                }
                                Err(Error::Timeout) => {
                                    next_input = Some(SerialApiMachineInput::Timeout);
                                }
                                Err(_) => {
                                    panic!("Unexpected internal error while waiting for ACK");
                                }
                            }
                        }
                        SerialApiMachineState::WaitingForResponse => {
                            match awaited_response
                                .take()
                                .expect("Response awaiter already consumed")
                                .try_await()
                                .await
                            {
                                Ok(response) if response.is_ok() => {
                                    next_input = Some(SerialApiMachineInput::Response(response));
                                }
                                Ok(response) => {
                                    next_input = Some(SerialApiMachineInput::ResponseNOK(response));
                                }
                                Err(Error::Timeout) => {
                                    next_input = Some(SerialApiMachineInput::Timeout);
                                }
                                Err(_) => {
                                    panic!("Unexpected internal error while waiting for response");
                                }
                            }
                        }
                        SerialApiMachineState::WaitingForCallback => {
                            match awaited_callback
                                .take()
                                .expect("Callback awaiter already consumed")
                                .try_await()
                                .await
                            {
                                Ok(callback) if callback.is_ok() => {
                                    next_input = Some(SerialApiMachineInput::Callback(callback));
                                }
                                Ok(callback) => {
                                    next_input = Some(SerialApiMachineInput::CallbackNOK(callback));
                                }
                                Err(Error::Timeout) => {
                                    next_input = Some(SerialApiMachineInput::Timeout);
                                }
                                Err(_) => {
                                    panic!("Unexpected internal error while waiting for callback");
                                }
                            }
                        }
                        SerialApiMachineState::Done(_) => (),
                    }
                }
            } else {
                println!("WARN: IDLE in Serial API machine - no input");
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        // Wait for the machine to finish
        let final_state = state_machine.state();

        match final_state {
            SerialApiMachineState::Done(s) => Ok(s.clone()),
            _ => panic!(
                "Serial API machine finished with invalid state {:?}",
                final_state
            ),
        }
    }

    pub fn log(&self) -> DriverLogger {
        DriverLogger::new(self.storage.logger().clone())
    }

    pub fn controller_log(&self) -> ControllerLogger {
        ControllerLogger::new(self.storage.logger().clone())
    }

    pub fn node_log(&self, node_id: NodeId, endpoint: EndpointIndex) -> NodeLogger {
        NodeLogger::new(self.storage.logger().clone(), node_id, endpoint)
    }

    pub fn security_manager(&self) -> Option<SecurityManager> {
        if let Some(storage) = self.storage.security_manager().as_ref() {
            Some(SecurityManager::new(storage.clone()))
        } else {
            None
        }
    }
}
