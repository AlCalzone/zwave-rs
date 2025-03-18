use crate::error::{Error, Result};
use crate::{
    expect_controller_command_result, ControllerCommandResult, ExecControllerCommandError, ExecControllerCommandOptions, ExecControllerCommandResult
};
use futures::channel::{mpsc, oneshot};
use futures::{select_biased, FutureExt, StreamExt};
use serial_api_machine::{
    SerialApiMachine, SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineResult,
    SerialApiMachineState,
};
use zwave_serial::command::{GetControllerVersionRequest, GetControllerVersionResponse};
use std::time::{Duration, Instant};
use zwave_cc::prelude::*;
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::util::MaybeSleep;
use zwave_core::{log::Loglevel, parse::Parsable, util::now};
use zwave_logging::loggers::driver2::DriverLogger2;
use zwave_logging::LocalImmutableLogger;
use zwave_logging::{loggers::serial2::SerialLogger2, Direction, LogInfo};
use zwave_serial::prelude::{Command, CommandBase, CommandEncodingContext, CommandParsingContext};
use zwave_serial::{
    command::AsCommandRaw,
    frame::{ControlFlow, RawSerialFrame, SerialFrame},
    prelude::{CommandRaw, CommandRequest},
};

mod awaited;
mod serial_api_machine;

pub struct RuntimeAdapter {
    pub serial_in: mpsc::Receiver<RawSerialFrame>,
    pub serial_out: mpsc::Sender<RawSerialFrame>,
    pub logs: mpsc::Sender<(LogInfo, Loglevel)>,
}

pub trait ExecutableCommand: CommandRequest + AsCommandRaw {}
impl<T> ExecutableCommand for T where T: CommandRequest + AsCommandRaw {}

struct SerialApiCommandState {
    command: Box<dyn ExecutableCommand>,
    timeout: Option<Instant>,
    expects_response: bool,
    expects_callback: bool,
    machine: SerialApiMachine,
    callback: Option<oneshot::Sender<Result<SerialApiMachineResult>>>,
}

/// A runtime- and IO agnostic adapter for serial ports.
/// Deals with parsing, serializing and logging serial frames,
/// but has to be driven by a runtime.
pub struct Driver2 {
    serial_in: mpsc::Receiver<RawSerialFrame>,
    serial_out: mpsc::Sender<RawSerialFrame>,
    log_queue: mpsc::Sender<(LogInfo, Loglevel)>,

    /// The serial API command that's currently being executed
    serial_api_command: Option<SerialApiCommandState>,

    input_tx: mpsc::Sender<DriverInput>,
    input_rx: mpsc::Receiver<DriverInput>,
}

pub struct Driver2Api {
    cmd_tx: mpsc::Sender<DriverInput>,
}

impl Driver2 {
    pub fn with_api(channels: RuntimeAdapter) -> (Self, Driver2Api) {
        let (input_tx, input_rx) = mpsc::channel(16);

        let driver = Self {
            serial_in: channels.serial_in,
            serial_out: channels.serial_out,
            log_queue: channels.logs,
            input_tx: input_tx.clone(),
            input_rx,

            serial_api_command: None,
        };

        let api = Driver2Api { cmd_tx: input_tx };

        (driver, api)
    }

    pub fn driver_log(&self) -> DriverLogger2 {
        DriverLogger2::new(self)
    }

    pub fn serial_log(&self) -> SerialLogger2 {
        SerialLogger2::new(self)
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
        {
            let driver_logger = self.driver_log();
            driver_logger.logo();
            driver_logger.info(|| "version 0.0.1-alpha");
            driver_logger.info(|| "");
            // driver_logger.info(|| format!("opening serial port {}", PORT));
        }

        loop {
            // We may or may not have a timeout to wait for. Construct a MaybeSleep to deal with this.
            let serial_api_timeout_duration = self
                .serial_api_command
                .as_ref()
                .and_then(|cmd| cmd.timeout)
                .and_then(|i| i.checked_duration_since(Instant::now()));
            let serial_api_sleep = MaybeSleep::new(serial_api_timeout_duration);

            select_biased! {
                // Handle incoming frames
                frame = self.serial_in.next() => {
                    if let Some(frame) = frame {
                        self.handle_serial_frame(frame);
                    }
                },
                // before inputs
                input = self.input_rx.next() => {
                    if let Some(input) = input {
                        self.handle_input(input);
                    }
                },
                // before timeouts
                _ = serial_api_sleep.fuse() => {
                    self.try_advance_serial_api_machine(SerialApiMachineInput::Timeout);
                }
            }
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

        self.serial_out
            .try_send(frame)
            .expect("failed to send frame to runtime");
    }

    fn queue_input(&self, input: DriverInput) {
        self.input_tx
            .clone()
            .try_send(input)
            .expect("Failed to queue driver input");
    }

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
                    timeout: None,
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

                // TODO: What else to do with this frame?
                #[expect(clippy::needless_return)]
                return;
            }
            SerialFrame::Command(raw) => {
                // Try to convert it into an actual command
                let cmd = {
                    // FIXME:
                    // self.get_command_parsing_context();
                    let ctx = CommandParsingContext::default();
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

    // Passes the input to the running serial API machine and returns whether it was handled
    fn try_advance_serial_api_machine(&mut self, input: SerialApiMachineInput) -> bool {
        let Some(SerialApiCommandState {
            // ref command,
            ref mut timeout,
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

        match machine.state() {
            SerialApiMachineState::WaitingForACK => {
                *timeout = Instant::now().checked_add(Duration::from_millis(1600));
            }

            // FIXME: Set better timeouts
            SerialApiMachineState::WaitingForResponse => {
                *timeout = Instant::now().checked_add(Duration::from_millis(10000));
            }

            SerialApiMachineState::WaitingForCallback => {
                *timeout = Instant::now().checked_add(Duration::from_millis(30000));
            }

            SerialApiMachineState::Done(result) => {
                callback
                    .take()
                    .expect("Serial API command callback already consumed")
                    .send(Ok(result.clone()))
                    .expect("Failed to send Serial API command result");
                self.serial_api_command = None;
            }

            _ => {}
        }

        true
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

impl Driver2Api {
    fn dispatch(&self, input: DriverInput) {
        self.cmd_tx
            .clone()
            .try_send(input)
            .expect("Failed to dispatch command");
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
        self.dispatch(cmd);

        rx.await.expect("Failed to receive command result")
    }

    // FIXME: Assert that the driver is ready for this command
    pub async fn exec_controller_command<C>(
        &mut self,
        command: C,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ExecControllerCommandResult<Option<Command>>
    where
        C: CommandRequest + AsCommandRaw + Into<Command> + Clone + 'static,
    {
        // FIXME:
        // let options = match options {
        //     Some(options) => options.clone(),
        //     None => Default::default(),
        // };

        // let supported = self.supports_function(command.function_type());
        // if options.enforce_support && !supported {
        //     return Err(ExecControllerCommandError::Unsupported(format!(
        //         "{:?}",
        //         command.function_type()
        //     )));
        // }

        let result = self.execute_serial_api_command(command).await;
        // TODO: Handle retrying etc.
        match result {
            Ok(SerialApiMachineResult::Success(command)) => Ok(command),
            Ok(result) => Err(result.into()),
            Err(e) => Err(ExecControllerCommandError::Unexpected(format!(
                "unexpected error in execute_serial_api_command: {:?}",
                e
            ))),
        }
    }

    pub async fn get_controller_version(
        &mut self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerVersionResponse> {
        // self.controller_log()
        //     .info(|| "querying controller version info...");
        let response = self
            .exec_controller_command(GetControllerVersionRequest::default(), options)
            .await;

        let version_info =
            expect_controller_command_result!(response, GetControllerVersionResponse);

        // if self.controller_log().level() < Loglevel::Debug {
        //     self.controller_log().info(|| {
        //         LogPayloadText::new("received controller version info:")
        //             .with_nested(version_info.to_log_payload())
        //     });
        // }

        // FIXME: Store SDK version here too

        Ok(version_info)
    }

}

impl LocalImmutableLogger for Driver2 {
    fn log(&self, log: LogInfo, level: Loglevel) {
        let _ = self.log_queue.clone().try_send((log, level));
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
