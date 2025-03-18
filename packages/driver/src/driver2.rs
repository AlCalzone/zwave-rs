use crate::error::{Error, Result};
use crate::DriverOptions;
use awaited::{AwaitedRef, AwaitedRegistry, Predicate};
use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use futures::{
    channel::{mpsc, oneshot},
    future::{BoxFuture, LocalBoxFuture},
    task::{LocalSpawn, LocalSpawnExt},
};
use petgraph::data;
use serial_api_machine::{
    SerialApiMachine, SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineResult,
    SerialApiMachineState,
};
use std::sync::Mutex;
use std::{
    collections::VecDeque,
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};
use zwave_cc::prelude::*;
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::{
    hex_bytes,
    log::Loglevel,
    parse::Parsable,
    prelude::{NodeId, NodeIdType, Serializable},
    util::now,
};
use zwave_logging::{loggers::serial2::SerialLogger2, Direction, LogInfo, Logger};
use zwave_logging::{ImmutableLogger, LocalImmutableLogger};
use zwave_serial::prelude::{CommandBase, CommandEncodingContext, CommandParsingContext};
use zwave_serial::{
    command::AsCommandRaw,
    frame::{ControlFlow, RawSerialFrame, SerialFrame},
    prelude::{Command, CommandRaw, CommandRequest},
};

mod awaited;
mod serial_api_machine;

const BUFFER_SIZE: usize = 256;

pub struct RuntimeAdapter {
    pub serial_in: mpsc::Receiver<RawSerialFrame>,
    pub serial_out: mpsc::Sender<RawSerialFrame>,
    pub logs: mpsc::Sender<LogInfo>,
}

// pub struct SerialChannels;

// impl SerialChannels {
//     pub fn new_split() -> (SerialChannelsDriverSide, SerialChannelsRuntimeSide) {
//         let (serial_in_tx, serial_in_rx) = mpsc::channel(16);
//         let (serial_out_tx, serial_out_rx) = mpsc::channel(16);

//         (
//             SerialChannelsDriverSide {
//                 serial_in: serial_in_rx,
//                 serial_out: serial_out_tx,
//             },
//             SerialChannelsRuntimeSide {
//                 serial_in: serial_in_tx,
//                 serial_out: serial_out_rx,
//             },
//         )
//     }
// }

// pub struct SerialChannelsDriverSide {
//     pub serial_in: mpsc::Receiver<RawSerialFrame>,
//     pub serial_out: mpsc::Sender<RawSerialFrame>,
// }

// pub struct SerialChannelsRuntimeSide {
//     pub serial_in: mpsc::Sender<RawSerialFrame>,
//     pub serial_out: mpsc::Receiver<RawSerialFrame>,
// }

pub trait Runtime: Send + Sync {
    fn spawn(
        &self,
        future: LocalBoxFuture<'static, ()>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn sleep(&self, duration: std::time::Duration) -> BoxFuture<'static, ()>;

    // // fn write_serial(&mut self, data: Bytes);
    // fn log(&self, log: LogInfo, level: Loglevel);
}

// pub struct RuntimeAdapter {
//     buffered_transmits: Mutex<VecDeque<Bytes>>,
//     buffered_events: Mutex<VecDeque<DriverEvent>>,
// }

// impl RuntimeAdapter {
//     pub fn new() -> Self {
//         Self {
//             buffered_transmits: Mutex::new(VecDeque::new()),
//             buffered_events: Mutex::new(VecDeque::new()),
//         }
//     }

//     pub fn queue_transmit(&self, data: Bytes) {
//         let mut queue = self.buffered_transmits.lock().unwrap();
//         queue.push_back(data);
//     }

//     pub fn poll_transmit(&self) -> Option<Bytes> {
//         let mut queue = self.buffered_transmits.lock().unwrap();
//         queue.pop_front()
//     }

//     pub fn queue_event(&self, event: DriverEvent) {
//         let mut queue = self.buffered_events.lock().unwrap();
//         queue.push_back(event);
//     }

//     pub fn poll_event(&self) -> Option<DriverEvent> {
//         let mut queue = self.buffered_events.lock().unwrap();
//         queue.pop_front()
//     }

//     fn queue_input(&self, input: DriverInput) {
//         self.queue_event(DriverEvent::Input { input });
//     }

//     pub fn serial_log(&self) -> SerialLogger2 {
//         SerialLogger2::new(self)
//     }

//     /// Handles a frame that was written to the input buffer
//     /// This should typically be handled before any other events,
//     /// so the Z-Wave module can go back to do what it was doing
//     pub fn handle_serial_data(&self, data: &mut BytesMut) {
//         while let Some(frame) = RawSerialFrame::parse_mut_or_reserve(data) {
//             match frame {
//                 RawSerialFrame::ControlFlow(byte) => {
//                     self.serial_log().control_flow(byte, Direction::Inbound);
//                     self.queue_input(DriverInput::Receive {
//                         frame: SerialFrame::ControlFlow(byte),
//                     });
//                 }
//                 RawSerialFrame::Data(mut bytes) => {
//                     self.serial_log().data(&bytes, Direction::Inbound);
//                     // Try to parse the frame
//                     match CommandRaw::parse(&mut bytes) {
//                         Ok(raw) => {
//                             // The first step of parsing was successful, ACK the frame
//                             self.queue_transmit(
//                                 RawSerialFrame::ControlFlow(ControlFlow::ACK).as_bytes(),
//                             );
//                             self.queue_input(DriverInput::Receive {
//                                 frame: SerialFrame::Command(raw),
//                             });
//                         }
//                         Err(e) => {
//                             println!("{} error: {}", now(), e);
//                             // Parsing failed, this means we've received garbage after all
//                             // Try to re-synchronize with the Z-Wave module
//                             self.queue_transmit(
//                                 RawSerialFrame::ControlFlow(ControlFlow::NAK).as_bytes(),
//                             );
//                         }
//                     }
//                 }
//                 RawSerialFrame::Garbage(bytes) => {
//                     self.serial_log().discarded(&bytes);
//                     // Try to re-synchronize with the Z-Wave module
//                     self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::NAK).as_bytes());
//                 }
//             }
//         }
//     }
// }

// impl Default for RuntimeAdapter {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl LocalImmutableLogger for RuntimeAdapter {
//     fn log(&self, log: LogInfo, level: Loglevel) {
//         self.queue_event(DriverEvent::Log { log, level });
//     }

//     fn log_level(&self) -> Loglevel {
//         Loglevel::Debug
//     }

//     fn set_log_level(&self, level: Loglevel) {
//         todo!()
//     }
// }

pub trait ExecutableCommand: CommandRequest + AsCommandRaw {}

impl<T> ExecutableCommand for T where T: CommandRequest + AsCommandRaw {}

struct SerialApiCommandState {
    command: Box<dyn ExecutableCommand>,
    expects_response: bool,
    expects_callback: bool,
    machine: SerialApiMachine,
    callback: Option<oneshot::Sender<Result<SerialApiMachineResult>>>,
}

/// A runtime- and IO agnostic adapter for serial ports.
/// Deals with parsing, serializing and logging serial frames,
/// but has to be driven by a runtime.
pub struct Driver2 {
    // buffered_transmits: VecDeque<RawSerialFrame>,
    buffered_events: VecDeque<DriverEvent>,

    rt: Arc<dyn Runtime>,
    awaited_control_flow_frames: Arc<AwaitedRegistry<ControlFlow>>,
    awaited_commands: Arc<AwaitedRegistry<Command>>,
    awaited_ccs: Arc<AwaitedRegistry<WithAddress<CC>>>,

    serial_in: mpsc::Receiver<RawSerialFrame>,
    serial_out: mpsc::Sender<RawSerialFrame>,
    log_queue: mpsc::Sender<LogInfo>,

    serial_api_command: Option<SerialApiCommandState>,

    input_queue: (mpsc::Sender<DriverInput>, mpsc::Receiver<DriverInput>),
}

impl Driver2 {
    pub fn new(rt: impl Runtime + 'static, channels: RuntimeAdapter) -> Self {
        Self {
            rt: Arc::new(rt),
            // buffered_transmits: VecDeque::new(),
            buffered_events: VecDeque::new(),
            // rt_adapter: Arc::new(RuntimeAdapter::default()),
            awaited_control_flow_frames: Arc::new(AwaitedRegistry::default()),
            awaited_commands: Arc::new(AwaitedRegistry::default()),
            awaited_ccs: Arc::new(AwaitedRegistry::default()),

            serial_in: channels.serial_in,
            serial_out: channels.serial_out,
            log_queue: channels.logs,
            input_queue: mpsc::channel(16),

            serial_api_command: None,
        }
    }

    pub fn serial_log(&self) -> SerialLogger2 {
        // self.rt_adapter.serial_log()
        SerialLogger2::new(self)
    }

    // pub fn serial_in(&self) -> mpsc::Sender<RawSerialFrame> {
    //     self.serial_in.0.clone()
    // }

    pub fn input_sender(&self) -> mpsc::Sender<DriverInput> {
        self.input_queue.0.clone()
    }

    /// Handles a frame that was written to the input buffer
    /// This should typically be handled before any other events,
    /// so the Z-Wave module can go back to do what it was doing
    pub fn handle_serial_frame(&mut self, frame: RawSerialFrame) {
        match frame {
            RawSerialFrame::ControlFlow(byte) => {
                self.serial_log().control_flow(byte, Direction::Inbound);
                self.queue_input(DriverInput::Receive {
                    frame: SerialFrame::ControlFlow(byte),
                });
            }
            RawSerialFrame::Data(mut bytes) => {
                self.serial_log().data(&bytes, Direction::Inbound);
                // Try to parse the frame
                match CommandRaw::parse(&mut bytes) {
                    Ok(raw) => {
                        // The first step of parsing was successful, ACK the frame
                        self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::ACK));
                        self.queue_input(DriverInput::Receive {
                            frame: SerialFrame::Command(raw),
                        });
                    }
                    Err(e) => {
                        println!("{} error: {}", now(), e);
                        // Parsing failed, this means we've received garbage after all
                        // Try to re-synchronize with the Z-Wave module
                        self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::NAK));
                    }
                }
            }
            RawSerialFrame::Garbage(bytes) => {
                self.serial_log().discarded(&bytes);
                // Try to re-synchronize with the Z-Wave module
                self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::NAK));
            }
        }
    }

    pub async fn run(&mut self) {
        loop {
            // Handle all incoming serial frames
            while let Ok(Some(frame)) = self.serial_in.try_next() {
                self.handle_serial_frame(frame);
            }

            // // If the driver has something to transmit, do that before handling events
            // while let Some(data) = self.buffered_transmits.pop_front() {
            //     self.serial_out
            //         .send(data)
            //         .await
            //         .expect("failed to send serial data");
            // }

            if let Ok(Some(input)) = self.input_queue.1.try_next() {
                self.handle_input(input);
                continue;
            }

            // // Check if an event needs to be handled
            // if let Some(event) = self.driver.poll_event() {
            //     match event {
            //         DriverEvent::Log { log, level } => {
            //             self.logger.log(log, level);
            //         }
            //         DriverEvent::Input { input } => {
            //             inputs.push_back(input);
            //         }
            //     }
            //     continue;
            // }

            // // Pass queued events to the driver
            // if let Some(input) = inputs.pop_front() {
            //     self.driver.handle_input(input);
            //     continue;
            // }

            // Event loop is empty, sleep for a bit
            tokio::time::sleep(Duration::from_millis(10)).await;
            // thread::sleep(Duration::from_millis(10));
        }
    }

    fn queue_transmit(&mut self, frame: RawSerialFrame) {
        match &frame {
            RawSerialFrame::Data(data) => {
                self.serial_log().data(data, Direction::Outbound);
            }
            RawSerialFrame::ControlFlow(byte) => {
                self.serial_log().control_flow(*byte, Direction::Outbound);
            }
            _ => {}
        }
        // let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        // frame.serialize(&mut buffer);
        // let output = buffer.split().freeze();

        self.serial_out
            .try_send(frame)
            .expect("failed to send frame to runtime");
    }

    // fn queue_event(&self, event: DriverEvent) {
    //     self.rt_adapter.queue_event(event);
    // }

    fn queue_input(&self, input: DriverInput) {
        self.input_queue
            .0
            .clone()
            .try_send(input)
            .expect("Failed to queue driver input");
    }

    // /// Returns a frame that is waiting for transmission, if any
    // pub fn poll_transmit(&self) -> Option<Bytes> {
    //     self.rt_adapter.poll_transmit()
    // }

    // /// Returns the timestamp of the next timeout. The caller should
    // /// call `handle_timeout` when the time is reached.
    // pub fn poll_timeout(&self) -> Option<Instant> {
    //     // FIXME: Implement timeouts
    //     None
    // }

    // /// Notifies the driver that the time has advanced to `now`
    // pub fn handle_timeout(&mut self, now: Instant) {
    //     // FIXME: Implement timeouts
    // }

    // /// Returns a pending event that should be handled by the caller, if any
    // pub fn poll_event(&self) -> Option<DriverEvent> {
    //     self.rt_adapter.poll_event()
    // }

    /// Passes an input that the driver needs to handle
    fn handle_input(&mut self, input: DriverInput) {
        match input {
            DriverInput::Transmit { frame } => {
                self.queue_transmit(frame.into());
            }
            DriverInput::Receive { frame } => {
                self.handle_frame(frame);
            }
            DriverInput::ExecCommand { command, callback } => {
                // FIXME: handle busy state

                // Set up state machine and interpreter
                let machine = SerialApiMachine::new();

                // TODO:
                // // Give the command a callback ID if it needs one
                // if command.needs_callback_id() && command.callback_id().is_none() {
                //     command.set_callback_id(Some(self.get_next_callback_id().await?));
                // }

                let expects_response = command.expects_response();
                let expects_callback = command.expects_callback();

                let ctx = CommandEncodingContext::default();
                let raw = command.as_raw(&ctx);
                let frame = SerialFrame::Command(raw);

                self.serial_api_command = Some(SerialApiCommandState {
                    command,
                    expects_response,
                    expects_callback,
                    machine,
                    callback: Some(callback),
                });
                // FIXME: This is unnecessary
                self.try_advance_serial_api_machine(SerialApiMachineInput::Start);

                self.queue_transmit(frame.into());

                self.try_advance_serial_api_machine(SerialApiMachineInput::FrameSent);
            }
        }
    }

    fn handle_frame(&mut self, frame: SerialFrame) {
        match frame {
            SerialFrame::ControlFlow(control_flow) => {
                // Forward control flow frames to the state machine if it's waiting for an ACK
                if let Some(SerialApiCommandState { machine, .. }) = &self.serial_api_command {
                    if *machine.state() == SerialApiMachineState::WaitingForACK {
                        println!("received {} while waiting for ACK", control_flow);
                        let handled = match control_flow {
                            ControlFlow::ACK => {
                                self.try_advance_serial_api_machine(SerialApiMachineInput::ACK)
                            }
                            ControlFlow::NAK => {
                                self.try_advance_serial_api_machine(SerialApiMachineInput::NAK)
                            }
                            ControlFlow::CAN => {
                                self.try_advance_serial_api_machine(SerialApiMachineInput::CAN)
                            }
                        };
                        if handled {
                            return;
                        }
                    }
                }

                // TODO: What else to do with this?
                return;
            }
            SerialFrame::Command(raw) => {
                // Try to convert it into an actual command
                let mut cmd = {
                    let ctx = CommandParsingContext::default(); // self.get_command_parsing_context();
                    match zwave_serial::command::Command::try_from_raw(raw, ctx) {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            println!("{} failed to decode CommandRaw: {}", now(), e);
                            // TODO: Handle misformatted frames
                            return;
                        }
                    }
                };

                // Check if this is an expected response or callback
                if let Some(SerialApiCommandState {
                    command, machine, ..
                }) = &self.serial_api_command
                {
                    match machine.state() {
                        SerialApiMachineState::WaitingForResponse
                            if command.test_response(&cmd) =>
                        {
                            println!("received matching response");
                            self.try_advance_serial_api_machine(if cmd.is_ok() {
                                SerialApiMachineInput::Response(cmd)
                            } else {
                                SerialApiMachineInput::ResponseNOK(cmd)
                            });
                            return;
                        }
                        SerialApiMachineState::WaitingForCallback
                            if command.test_callback(&cmd) =>
                        {
                            println!("received matching callback");
                            self.try_advance_serial_api_machine(if cmd.is_ok() {
                                SerialApiMachineInput::Callback(cmd)
                            } else {
                                SerialApiMachineInput::CallbackNOK(cmd)
                            });
                            return;
                        }
                        _ => {}
                    }
                }

                todo!("handle received command: {:?}", cmd);
            }
            // Not much we can do with a raw frame at this point
            _ => {
                todo!("handle received frame: {:?}", frame);
            }
        }
    }

    async fn await_control_flow_frame(
        &self,
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>,
    ) -> Result<ControlFlow> {
        self.awaited_control_flow_frames
            .add(predicate, timeout)
            .try_await()
            .await
    }

    // Passes the input to the running serial API machine and returns whether it was handled
    fn try_advance_serial_api_machine(&mut self, input: SerialApiMachineInput) -> bool {
        let Some(SerialApiCommandState {
            // ref command,
            expects_response,
            expects_callback,
            ref mut machine,
            ref mut callback,
            ..
        }) = self.serial_api_command
        else {
            return false;
        };

        if machine.done() {
            return false;
        }

        // let expects_response = command.expects_response();
        // let expects_callback = command.expects_callback();
        // let evaluate_condition =
        //     Box::new(
        //         move |condition: SerialApiMachineCondition| match condition {
        //             SerialApiMachineCondition::ExpectsResponse => expects_response,
        //             SerialApiMachineCondition::ExpectsCallback => expects_callback,
        //         },
        //     );

        let Some(transition) =
            machine.next(
                input,
                |condition: SerialApiMachineCondition| match condition {
                    SerialApiMachineCondition::ExpectsResponse => expects_response,
                    SerialApiMachineCondition::ExpectsCallback => expects_callback,
                },
            )
        else {
            return false;
        };

        // Transition to the new state
        machine.transition(transition.new_state());

        if let SerialApiMachineState::Done(result) = machine.state() {
            callback
                .take()
                .expect("Serial API command callback already consumed")
                .send(Ok(result.clone()))
                .expect("Failed to send Serial API command result");
            self.serial_api_command = None;
        }

        // FIXME: Set up timeouts while waiting

        // // Now check what needs to be done in the new state
        // match machine.state() {
        //     SerialApiMachineState::Initial => return false,
        //     SerialApiMachineState::Sending => {
        //         // TODO:
        //         // let ctx = CommandEncodingContext::builder()
        //         //     .own_node_id(self.own_node_id())
        //         //     .node_id_type(self.storage.node_id_type())
        //         //     .sdk_version(self.storage.sdk_version())
        //         //     .build();
        //         let ctx = CommandEncodingContext::default();

        //         let raw = command.as_raw(&ctx);
        //         let frame = SerialFrame::Command(raw);
        //         self.queue_transmit(frame.into());

        //         // TODO: Logs

        //         // Advance the state machine once more
        //         return self.try_advance_serial_api_machine(SerialApiMachineInput::FrameSent);

        //         // // Send the command to the controller
        //         // awaited_ack = Some(
        //         //     self.write_serial(frame.into())
        //         //         .await
        //         //         .expect("write_serial failed"),
        //         // );
        //         // // and notify the state machine
        //         // next_input = Some(SerialApiMachineInput::FrameSent);

        //         // // And log the command information if this was a command
        //         // let node_id = match Into::<Command>::into(command.clone()) {
        //         //     // FIXME: Extract the endpoint index aswell
        //         //     Command::SendDataRequest(cmd) => Some(cmd.node_id),
        //         //     _ => None,
        //         // };

        //         // if let Some(node_id) = node_id {
        //         //     self.node_log(node_id, EndpointIndex::Root)
        //         //         .command(&command, Direction::Outbound);
        //         // } else {
        //         //     self.controller_log().command(&command, Direction::Outbound);
        //         // }
        //     } // FIXME: Set up timeouts while waiting

        //     // SerialApiMachineState::WaitingForACK => {
        //     //     // TODO: Set up timeout
        //     //     // // Wait for ACK, but also accept CAN and NAK
        //     //     // match awaited_ack
        //     //     //     .take()
        //     //     //     .expect("ACK awaiter already consumed")
        //     //     //     .try_await()
        //     //     //     .await
        //     //     // {
        //     //     //     Ok(frame) => {
        //     //     //         next_input = Some(match frame {
        //     //     //             ControlFlow::ACK => SerialApiMachineInput::ACK,
        //     //     //             ControlFlow::NAK => SerialApiMachineInput::NAK,
        //     //     //             ControlFlow::CAN => SerialApiMachineInput::CAN,
        //     //     //         });
        //     //     //     }
        //     //     //     Err(Error::Timeout) => {
        //     //     //         next_input = Some(SerialApiMachineInput::Timeout);
        //     //     //     }
        //     //     //     Err(_) => {
        //     //     //         panic!("Unexpected internal error while waiting for ACK");
        //     //     //     }
        //     //     // }
        //     // }
        //     // SerialApiMachineState::WaitingForResponse => {
        //     //     match awaited_response
        //     //         .take()
        //     //         .expect("Response awaiter already consumed")
        //     //         .try_await()
        //     //         .await
        //     //     {
        //     //         Ok(response) if response.is_ok() => {
        //     //             next_input = Some(SerialApiMachineInput::Response(response));
        //     //         }
        //     //         Ok(response) => {
        //     //             next_input = Some(SerialApiMachineInput::ResponseNOK(response));
        //     //         }
        //     //         Err(Error::Timeout) => {
        //     //             next_input = Some(SerialApiMachineInput::Timeout);
        //     //         }
        //     //         Err(_) => {
        //     //             panic!("Unexpected internal error while waiting for response");
        //     //         }
        //     //     }
        //     // }
        //     // SerialApiMachineState::WaitingForCallback => {
        //     //     match awaited_callback
        //     //         .take()
        //     //         .expect("Callback awaiter already consumed")
        //     //         .try_await()
        //     //         .await
        //     //     {
        //     //         Ok(callback) if callback.is_ok() => {
        //     //             next_input = Some(SerialApiMachineInput::Callback(callback));
        //     //         }
        //     //         Ok(callback) => {
        //     //             next_input = Some(SerialApiMachineInput::CallbackNOK(callback));
        //     //         }
        //     //         Err(Error::Timeout) => {
        //     //             next_input = Some(SerialApiMachineInput::Timeout);
        //     //         }
        //     //         Err(_) => {
        //     //             panic!("Unexpected internal error while waiting for callback");
        //     //         }
        //     //     }
        //     // }
        //     // SerialApiMachineState::Done(_) => (),
        //     _ => {}
        // }

        return true;
    }

    pub async fn execute_serial_api_command<C>(&self, command: C) -> Result<SerialApiMachineResult>
    where
        C: ExecutableCommand + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let cmd = DriverInput::ExecCommand {
            command: Box::new(command),
            callback: tx,
        };
        self.queue_input(cmd);

        rx.await.expect("Failed to receive command result")
    }
}

pub struct Driver2Api {
    cmd_tx: mpsc::Sender<DriverInput>,
}

impl Driver2Api {
    pub fn new(cmd_tx: mpsc::Sender<DriverInput>) -> Self {
        Self { cmd_tx }
    }

    pub async fn execute_serial_api_command<C>(
        &mut self,
        command: C,
    ) -> Result<SerialApiMachineResult>
    where
        C: ExecutableCommand + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let cmd = DriverInput::ExecCommand {
            command: Box::new(command),
            callback: tx,
        };
        self.cmd_tx.send(cmd).await.expect("Failed to send command");

        rx.await.expect("Failed to receive command result")
    }
}

impl LocalImmutableLogger for Driver2 {
    fn log(&self, log: LogInfo, level: Loglevel) {
        // self.log_queue.send(log).
        // self.rt.log(log, level);
        // println!("{}", log);
    }

    fn log_level(&self) -> Loglevel {
        Loglevel::Debug
    }

    fn set_log_level(&self, level: Loglevel) {
        todo!()
    }
}

pub enum DriverInput {
    Transmit {
        frame: SerialFrame,
    },
    /// Notify the application that a frame was received
    Receive {
        frame: SerialFrame,
    },
    ExecCommand {
        command: Box<dyn ExecutableCommand>,
        callback: oneshot::Sender<Result<SerialApiMachineResult>>,
    },
}

pub enum DriverEvent {
    // /// Log the given message
    // Log { log: LogInfo, level: Loglevel },
}
