use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;
use std::vec::Vec;

use zwave_core::prelude::*;
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::util::{now, MaybeSleep};
use zwave_serial::prelude::*;

use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::{ControlFlow, RawSerialFrame, SerialFrame};
use zwave_serial::serialport::SerialPort;

use crate::error::{Error, Result};

use tokio::sync::{broadcast, mpsc, oneshot, Mutex, Notify};
use tokio::task::JoinHandle;

use self::awaited::{AwaitedRegistry, Predicate};
use self::serial_api_machine::{
    SerialApiMachine, SerialApiMachineCondition, SerialApiMachineEffect, SerialApiMachineInput,
    SerialApiMachineResult, SerialApiMachineState,
};

mod awaited;
mod serial_api_machine;

type TaskCommandSender<T> = mpsc::Sender<(T, oneshot::Sender<()>)>;
type TaskCommandReceiver<T> = mpsc::Receiver<(T, oneshot::Sender<()>)>;

type SerialFrameEmitter = broadcast::Sender<SerialFrame>;
type SerialListener = broadcast::Receiver<SerialFrame>;

type CommandHandler = Box<dyn Fn(Command) -> bool + Send + Sync>;

struct DriverInner {
    command_handlers: Mutex<Vec<CommandHandler>>,
    awaited_control_flow_frames: AwaitedRegistry<ControlFlow>,
    awaited_commands: AwaitedRegistry<Command>,
}

#[allow(dead_code)]
pub struct Driver {
    inner: Arc<DriverInner>,

    serial_task: JoinHandle<()>,
    main_task: JoinHandle<()>,
    main_cmd: MainTaskCommandSender,
    main_task_shutdown: Arc<Notify>,
    serial_cmd: SerialTaskCommandSender,
    serial_listener: SerialListener,
    serial_task_shutdown: Arc<Notify>,
    // command_handlers: Arc<Mutex<Vec<SerialCommandHandlerSender>>>,
    // command_handlers: Arc<Mutex<Vec<CommandHandler>>>,
}

impl Driver {
    pub fn new(path: &str) -> Self {
        // The serial task owns the serial port. All communication needs to go through that task.
        let port = SerialPort::new(path).unwrap();
        // To control it, we send a thread command along with a "callback" oneshot channel to the task.
        let (serial_cmd_tx, serial_cmd_rx) =
            mpsc::channel::<(SerialTaskCommand, oneshot::Sender<()>)>(100);
        // The listener is used to receive frames from the serial port
        let (serial_listener_tx, serial_listener_rx) = broadcast::channel::<SerialFrame>(100);
        let serial_task_shutdown = Arc::new(Notify::new());
        let serial_task_shutdown2 = serial_task_shutdown.clone();

        // The main logic happens in another task that owns the internal state.
        // To control it, we need another channel.
        let (main_cmd_tx, main_cmd_rx) =
            mpsc::channel::<(MainTaskCommand, oneshot::Sender<()>)>(100);
        let main_serial_cmd = serial_cmd_tx.clone();
        let main_serial_listener = serial_listener_tx.subscribe();
        let main_task_shutdown = Arc::new(Notify::new());
        let main_task_shutdown2 = main_task_shutdown.clone();

        // let command_handlers: Vec<SerialCommandHandlerSender> = Vec::new();
        // let command_handlers = Arc::new(Mutex::new(command_handlers));
        let command_handlers: Vec<CommandHandler> = Vec::new();
        let command_handlers = Mutex::new(command_handlers);

        let awaited_control_flow_frames = AwaitedRegistry::default();
        let awaited_commands = AwaitedRegistry::default();

        let inner = DriverInner {
            command_handlers,
            awaited_control_flow_frames,
            awaited_commands,
        };
        let inner = Arc::new(inner);

        // Start the background task for the main logic
        let main_task = tokio::spawn(main_loop(
            inner.clone(),
            main_cmd_rx,
            main_task_shutdown2,
            main_serial_cmd,
            main_serial_listener,
        ));

        // Start the background task for the serial port communication
        let serial_task = tokio::spawn(serial_loop(
            port,
            serial_cmd_rx,
            serial_task_shutdown2,
            serial_listener_tx,
        ));

        Self {
            main_task,
            main_cmd: main_cmd_tx,
            main_task_shutdown,
            serial_task,
            serial_cmd: serial_cmd_tx,
            serial_task_shutdown,
            serial_listener: serial_listener_rx,
            inner,
        }
    }

    pub async fn write_serial(&self, frame: SerialFrame) -> Result<()> {
        exec_background_task(&self.serial_cmd, SerialTaskCommand::Send(frame)).await
    }

    pub async fn register_command_handler(&mut self, handler: CommandHandler) {
        let mut handlers = self.inner.command_handlers.lock().await;
        handlers.push(handler);
        println!("registered command handler, count: {}", handlers.len());
    }

    async fn await_control_flow_frame(
        &self,
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>,
    ) -> Option<(ControlFlow, oneshot::Sender<()>)> {
        // To await a control frame, we first register an awaiter
        let mut awaiter = self.inner.awaited_control_flow_frames.add(predicate); // self.register_control_frame_awaiter(predicate);

        // ...wait for it to be fulfilled or time out
        let sleep = MaybeSleep::new(timeout);
        tokio::select! {
            // We pass the entire result including the oneshot channel to the caller,
            // so that they can acknowledge the command when they handled it. This avoids
            // race conditions where the driver may attempt to handle the next serial frame
            // before it is expected.
            // FIXME: This entire setup is a bit awkward to use - abstract it away into an Acknowledged trait?
            result = awaiter.take_channel() => Some(result.unwrap()),
            _ = sleep => None,
        }
    }

    pub async fn await_command(
        &self,
        predicate: Predicate<Command>,
        timeout: Option<Duration>,
    ) -> Option<(Command, oneshot::Sender<()>)> {
        // To await a command, we first register an awaiter
        let mut awaiter = self.inner.awaited_commands.add(predicate); // self.register_command_awaiter(predicate);

        // ...wait for it to be fulfilled or time out
        let sleep = MaybeSleep::new(timeout);
        tokio::select! {
            // We pass the entire result including the oneshot channel to the caller,
            // so that they can acknowledge the command when they handled it. This avoids
            // race conditions where the driver may attempt to handle the next serial frame
            // before it is expected.
            result = awaiter.take_channel() => Some(result.unwrap()),
            _ = sleep => None,
        }
    }

    pub async fn execute_serial_api_command<C>(&self, command: C) -> Result<SerialApiMachineResult>
    where
        C: CommandRequest + Clone + 'static,
        SerialFrame: From<C>,
    {
        // Set up state machine and interpreter
        let mut state_machine = SerialApiMachine::new();

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
        let mut ack: Option<oneshot::Sender<()>> = None;
        while !state_machine.done() {
            if let Some(input) = next_input.take() {
                if let Some(transition) = state_machine.next(input, &evaluate_condition) {
                    let new_state = transition.new_state();

                    if let Some(effect) = transition.effect() {
                        println!("{} handling effect {:?}", now(), effect);
                        if let Some(ack) = ack.take() {
                            ack.send(()).unwrap();
                        }
                        match effect {
                            SerialApiMachineEffect::SendFrame => {
                                // FIXME: We should take a reference here and use try_into()
                                let frame = SerialFrame::from(command.clone());
                                // Send the command to the controller
                                self.write_serial(frame).await.unwrap();
                                // and notify the state machine
                                next_input = Some(SerialApiMachineInput::FrameSent);
                            }
                            SerialApiMachineEffect::AbortSending => {
                                todo!("Handle effect {:?}", effect)
                            }
                            SerialApiMachineEffect::WaitForACK => {
                                // Wait for ACK, but also accept CAN and NAK
                                let awaited = self
                                    .await_control_flow_frame(
                                        Box::new(|_| true),
                                        Some(Duration::from_millis(1600)),
                                    )
                                    .await;
                                let (frame, handled) = awaited
                                    .map_or_else(|| (None, None), |(f, h)| (Some(f), Some(h)));
                                // Notify the state machine about the result
                                next_input = Some(match frame {
                                    Some(ControlFlow::ACK) => SerialApiMachineInput::ACK,
                                    Some(ControlFlow::NAK) => SerialApiMachineInput::NAK,
                                    Some(ControlFlow::CAN) => SerialApiMachineInput::CAN,
                                    None => SerialApiMachineInput::Timeout,
                                });
                                ack = handled;
                            }
                            SerialApiMachineEffect::WaitForResponse => {
                                let command = command.clone();
                                let awaited = self
                                    .await_command(
                                        Box::new(move |cmd: &Command| command.test_response(cmd)),
                                        Some(Duration::from_millis(10000)),
                                    )
                                    .await;
                                let (response, handled) = awaited
                                    .map_or_else(|| (None, None), |(c, h)| (Some(c), Some(h)));
                                next_input = Some(match response {
                                    Some(response) if response.is_ok() => {
                                        SerialApiMachineInput::Response(response)
                                    }
                                    Some(response) => SerialApiMachineInput::ResponseNOK(response),
                                    None => SerialApiMachineInput::Timeout,
                                });
                                ack = handled;
                            }
                            SerialApiMachineEffect::WaitForCallback => {
                                let command = command.clone();
                                let awaited = self
                                    .await_command(
                                        Box::new(move |cmd: &Command| command.test_callback(cmd)),
                                        Some(Duration::from_millis(30000)),
                                    )
                                    .await;
                                let (callback, handled) = awaited
                                    .map_or_else(|| (None, None), |(c, h)| (Some(c), Some(h)));

                                next_input = Some(match callback {
                                    Some(callback) if callback.is_ok() => {
                                        SerialApiMachineInput::Callback(callback)
                                    }
                                    Some(callback) => SerialApiMachineInput::CallbackNOK(callback),
                                    None => SerialApiMachineInput::Timeout,
                                });
                                ack = handled;
                            }
                        }
                    } else {
                        match new_state {
                            SerialApiMachineState::Done(_) => (),
                            _ => {
                                println!("WARN: IDLE in Serial API machine - no effect");
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            }
                        };
                    }
                    state_machine.transition(new_state);
                }
            } else {
                println!("WARN: IDLE in Serial API machine - no input");
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        if let Some(ack) = ack.take() {
            ack.send(()).unwrap();
        }

        // Wait for the machine to finish
        let final_state = state_machine.state();

        match final_state {
            serial_api_machine::SerialApiMachineState::Done(s) => Ok(s.clone()),
            _ => panic!(
                "Serial API machine finished with invalid state {:?}",
                final_state
            ),
        }
    }
}

enum MainTaskCommand {}
type MainTaskCommandSender = TaskCommandSender<MainTaskCommand>;
type MainTaskCommandReceiver = TaskCommandReceiver<MainTaskCommand>;

async fn main_loop(
    inner: Arc<DriverInner>,
    mut cmd_rx: MainTaskCommandReceiver,
    shutdown: Arc<Notify>,
    serial_cmd: SerialTaskCommandSender,
    mut serial_listener: SerialListener,
    // command_handlers: Arc<Mutex<Vec<CommandHandler>>>,
) {
    loop {
        tokio::select! {
            // Make sure we don't read from the serial port if there is a potential command
            // waiting to set up a frame handler
            biased;

            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some((cmd, done)) = cmd_rx.recv() => main_loop_handle_command(&inner, cmd, done, &serial_cmd).await,

            // The serial port emitted a frame
            Ok(frame) = serial_listener.recv() => main_loop_handle_frame(&inner, frame, &serial_cmd).await
        }
    }

    println!("main task stopped")
}

async fn main_loop_handle_command(
    _inner: &Arc<DriverInner>,
    cmd: MainTaskCommand,
    _done: oneshot::Sender<()>,
    _serial_cmd: &SerialTaskCommandSender,
) {
    match cmd {
        _ => {} // Ignore other commands
    }
}

async fn main_loop_handle_frame(
    inner: &Arc<DriverInner>,
    frame: SerialFrame,
    _serial_cmd: &SerialTaskCommandSender,
) {
    // TODO: Consider if we need to always handle something here
    match &frame {
        SerialFrame::ControlFlow(cf) => {
            // If the awaited control-flow-frame registry has a matching awaiter,
            // remove it and send the frame through its channel
            if let Some(channel) = inner.awaited_control_flow_frames.take_matching(cf) {
                // Send the frame through the channel along with a callback oneshot to acknowledge it
                let (done_tx, done_rx) = oneshot::channel::<()>();
                channel.send((*cf, done_tx)).unwrap();
                done_rx.await.unwrap();
            }
        }
        SerialFrame::Command(cmd) => {
            // If the awaited command registry has a matching awaiter,
            // remove it and send the command through its channel
            if let Some(channel) = inner.awaited_commands.take_matching(cmd) {
                // Send the command through the channel along with a callback oneshot to acknowledge it
                let (done_tx, done_rx) = oneshot::channel::<()>();
                channel.send((cmd.clone(), done_tx)).unwrap();
                done_rx.await.unwrap();
            }
        }
        _ => {}
    }
    // tokio::time::sleep(Duration::from_millis(10)).await;
}

enum SerialTaskCommand {
    Send(SerialFrame),
}

type SerialTaskCommandSender = TaskCommandSender<SerialTaskCommand>;
type SerialTaskCommandReceiver = TaskCommandReceiver<SerialTaskCommand>;

async fn serial_loop(
    mut port: SerialPort,
    mut cmd_rx: SerialTaskCommandReceiver,
    shutdown: Arc<Notify>,
    frame_emitter: SerialFrameEmitter,
) {
    loop {
        // Whatever happens first gets handled first.
        tokio::select! {
            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some((cmd, done)) = cmd_rx.recv() => serial_loop_handle_command(&mut port, cmd, done).await,

            // We received a frame from the serial port
            Some(frame) = port.read() => serial_loop_handle_frame(&mut port, frame, &frame_emitter).await
        }
    }

    println!("serial task stopped")
}

async fn serial_loop_handle_command(
    port: &mut SerialPort,
    cmd: SerialTaskCommand,
    done: oneshot::Sender<()>,
) {
    #[allow(irrefutable_let_patterns)]
    if let SerialTaskCommand::Send(frame) = cmd {
        port.write(frame.try_into().unwrap()).await.unwrap();
        done.send(()).unwrap();
    }
}

async fn serial_loop_handle_frame(
    port: &mut SerialPort,
    frame: RawSerialFrame,
    frame_emitter: &SerialFrameEmitter,
) {
    let emit = match &frame {
        RawSerialFrame::Data(data) => {
            println!("{} << {}", now(), hex::encode(data));
            // Try to parse the frame
            match zwave_serial::command_raw::CommandRaw::parse(data) {
                Ok((_, raw)) => {
                    // The first step of parsing was successful, ACK the frame
                    port.write(RawSerialFrame::ControlFlow(ControlFlow::ACK))
                        .await
                        .unwrap();

                    // Now try to convert it into an actual command
                    match zwave_serial::command::Command::try_from(raw) {
                        Ok(cmd) => {
                            println!("{} received {:#?}", now(), cmd);
                            Some(SerialFrame::Command(cmd))
                        }
                        Err(e) => {
                            println!("{} error: {:?}", now(), e);
                            // TODO: Handle misformatted frames
                            None
                        }
                    }
                }
                Err(e) => {
                    println!("{} error: {:?}", now(), e);
                    // Parsing failed, this means we've received garbage after all
                    port.write(RawSerialFrame::ControlFlow(ControlFlow::NAK))
                        .await
                        .unwrap();
                    None
                }
            }
        }
        RawSerialFrame::Garbage(data) => {
            println!("{} xx: {}", now(), hex::encode(data));
            // After receiving garbage, try to re-sync by sending NAK
            port.write(RawSerialFrame::ControlFlow(ControlFlow::NAK))
                .await
                .unwrap();
            None
        }
        RawSerialFrame::ControlFlow(byte) => {
            println!("{} << {:?}", now(), &byte);
            Some(SerialFrame::ControlFlow(*byte))
        }
    };

    if let Some(frame) = emit {
        frame_emitter.send(frame).unwrap();
    }
}

async fn exec_background_task<T>(command_sender: &TaskCommandSender<T>, cmd: T) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    command_sender
        .send((cmd, tx))
        .await
        .map_err(|_| Error::Internal)?;
    rx.await.map_err(|_| Error::Internal)?;
    Ok(())
}

impl Drop for Driver {
    fn drop(&mut self) {
        // We need to stop the background tasks, otherwise they will stick around until the process exits
        self.serial_task_shutdown.notify_one();
        self.main_task_shutdown.notify_one();
    }
}
