use super::{
    SerialApiActor, SerialApiCommandState, SerialApiEvent, SerialApiInput, SerialApiMachine,
    SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineState,
};
use core::time::Duration;
use zwave_core::prelude::*;
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::util::MaybeSleep;
use zwave_core::{log::Loglevel, parse::Parsable};
use zwave_logging::{
    loggers::{controller::ControllerLogger, driver::DriverLogger, serial::SerialLogger},
    Direction, LocalImmutableLogger, LogInfo,
};
use zwave_pal::time::Instant;
use zwave_serial::frame::{ControlFlow, RawSerialFrame, SerialFrame};
use zwave_serial::prelude::*;

impl SerialApiActor {
    pub async fn run(&mut self) {
        {
            let driver_logger = self.driver_log();
            driver_logger.logo();
            driver_logger.info(|| "version 0.0.1-alpha");
            driver_logger.info(|| "");
        }

        loop {
            let serial_api_timeout_duration = self
                .serial_api_command
                .as_ref()
                .and_then(|cmd| cmd.timeout)
                .and_then(|i| i.checked_duration_since(Instant::now()));
            let serial_api_sleep = MaybeSleep::new(serial_api_timeout_duration);

            zwave_pal::select_biased! {
                frame = self.serial_in.recv() => {
                    if let Some(frame) = frame {
                        self.handle_serial_frame(frame);
                    }
                },
                input = self.input_rx.recv() => {
                    if let Some(input) = input {
                        self.handle_input(input);
                    }
                },
                _ = serial_api_sleep => {
                    self.try_advance_serial_api_machine(SerialApiMachineInput::Timeout);
                }
            }
        }
    }

    pub fn driver_log(&self) -> DriverLogger<'_> {
        DriverLogger::new(self)
    }

    pub fn serial_log(&self) -> SerialLogger<'_> {
        SerialLogger::new(self)
    }

    pub fn controller_log(&self) -> ControllerLogger<'_> {
        ControllerLogger::new(self)
    }

    pub fn handle_serial_frame(&mut self, frame: RawSerialFrame) {
        match frame {
            RawSerialFrame::ControlFlow(byte) => {
                self.serial_log().control_flow(byte, Direction::Inbound);
                self.queue_input(SerialApiInput::Receive {
                    frame: SerialFrame::ControlFlow(byte),
                });
            }
            RawSerialFrame::Data(mut bytes) => {
                self.serial_log().data(&bytes, Direction::Inbound);
                match CommandRaw::parse(&mut bytes) {
                    Ok(raw) => {
                        self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::ACK));
                        self.queue_input(SerialApiInput::Receive {
                            frame: SerialFrame::Command(raw),
                        });
                    }
                    Err(_e) => {
                        self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::NAK));
                    }
                }
            }
            RawSerialFrame::Garbage(bytes) => {
                self.serial_log().discarded(&bytes);
                self.queue_transmit(RawSerialFrame::ControlFlow(ControlFlow::NAK));
            }
        }
    }

    fn handle_input(&mut self, input: SerialApiInput) {
        match input {
            SerialApiInput::Transmit { frame } => {
                self.queue_transmit(frame.into());
            }
            SerialApiInput::Receive { frame } => {
                self.handle_frame(frame);
            }
            SerialApiInput::ExecCommand {
                mut command,
                callback,
            } => {
                let machine = SerialApiMachine::new();

                if command.needs_callback_id() && command.callback_id().is_none() {
                    command.set_callback_id(Some(self.get_next_callback_id()));
                }

                let expects_response = command.expects_response();
                let expects_callback = command.expects_callback();

                let ctx = CommandEncodingContext::builder()
                    .own_node_id(self.storage.own_node_id().get())
                    .node_id_type(self.storage.node_id_type().get())
                    .sdk_version(self.storage.sdk_version().get())
                    .build();
                let raw = command.as_raw(&ctx);
                let frame = SerialFrame::Command(raw);

                self.controller_log()
                    .command(command.as_ref(), Direction::Outbound);

                self.serial_api_command = Some(SerialApiCommandState {
                    command,
                    timeout: None,
                    expects_response,
                    expects_callback,
                    machine,
                    callback: Some(callback),
                });
                self.queue_transmit(frame.into());

                self.try_advance_serial_api_machine(SerialApiMachineInput::Start);
            }
            SerialApiInput::Log { log, level } => {
                self.log_queue
                    .try_send((log, level))
                    .expect("Failed to log message");
            }
        }
    }

    fn handle_frame(&mut self, frame: SerialFrame) {
        match frame {
            SerialFrame::ControlFlow(control_flow) => {
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

                #[expect(clippy::needless_return)]
                return;
            }
            SerialFrame::Command(raw) => {
                let cmd = {
                    let ctx = CommandParsingContext::builder()
                        .own_node_id(self.storage.own_node_id().get())
                        .node_id_type(self.storage.node_id_type().get())
                        .sdk_version(self.storage.sdk_version().get())
                        .build();
                    match zwave_serial::command::Command::try_from_raw(raw, ctx) {
                        Ok(cmd) => cmd,
                        Err(_e) => {
                            return;
                        }
                    }
                };

                if let Some(SerialApiCommandState {
                    command,
                    machine,
                    ..
                }) = &self.serial_api_command
                {
                    let input = match machine.state() {
                        SerialApiMachineState::WaitingForResponse
                            if command.test_response(&cmd) =>
                        {
                            if cmd.is_ok() {
                                Some(SerialApiMachineInput::Response(cmd.clone()))
                            } else {
                                Some(SerialApiMachineInput::ResponseNOK(cmd.clone()))
                            }
                        }
                        SerialApiMachineState::WaitingForCallback
                            if command.test_callback(&cmd) =>
                        {
                            if cmd.is_ok() {
                                Some(SerialApiMachineInput::Callback(cmd.clone()))
                            } else {
                                Some(SerialApiMachineInput::CallbackNOK(cmd.clone()))
                            }
                        }
                        _ => None,
                    };
                    if let Some(input) = input {
                        self.try_advance_serial_api_machine(input);
                        return;
                    }
                }

                self.queue_event(SerialApiEvent::Unsolicited { command: cmd });
            }
            _ => {
                todo!("handle received frame: {:?}", frame);
            }
        }
    }

    fn try_advance_serial_api_machine(&mut self, input: SerialApiMachineInput) -> bool {
        let Some(SerialApiCommandState {
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

        let Some(transition) = machine.next(
            input.clone(),
            |condition: SerialApiMachineCondition| match condition {
                SerialApiMachineCondition::ExpectsResponse => expects_response,
                SerialApiMachineCondition::ExpectsCallback => expects_callback,
            },
        ) else {
            return false;
        };

        machine.transition(transition.new_state());

        match machine.state() {
            SerialApiMachineState::WaitingForACK => {
                *timeout = Instant::now().checked_add(Duration::from_millis(1600));
            }
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

        if let SerialApiMachineInput::Response(cmd)
        | SerialApiMachineInput::ResponseNOK(cmd)
        | SerialApiMachineInput::Callback(cmd)
        | SerialApiMachineInput::CallbackNOK(cmd) = input
        {
            self.controller_log().command(&cmd, Direction::Inbound);
        }

        true
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
            .expect("failed to queue frame for transmit");
    }

    fn queue_input(&self, input: SerialApiInput) {
        self.input_tx
            .clone()
            .try_send(input)
            .expect("Failed to queue serial API input");
    }

    fn queue_event(&self, event: SerialApiEvent) {
        self.event_tx
            .clone()
            .try_send(event)
            .expect("Failed to queue serial API event");
    }

    fn get_next_callback_id(&mut self) -> u8 {
        self.callback_id.increment()
    }
}

impl LocalImmutableLogger for SerialApiActor {
    fn log(&self, log: LogInfo, level: Loglevel) {
        let _ = self.log_queue.clone().try_send((log, level));
    }

    fn log_level(&self) -> Loglevel {
        Loglevel::Debug
    }

    fn set_log_level(&self, _level: Loglevel) {
        todo!()
    }
}
