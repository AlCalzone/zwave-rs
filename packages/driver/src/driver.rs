use std::sync::Arc;
use std::time::Duration;

use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::util::now;
use zwave_core::wrapping_counter::WrappingCounter;
use zwave_core::{prelude::*, submodule};
use zwave_serial::prelude::*;

use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::{ControlFlow, RawSerialFrame, SerialFrame};
use zwave_serial::serialport::SerialPort;

use crate::error::{Error, Result};
use crate::Controller;

use tokio::sync::{broadcast, mpsc, oneshot, Notify};
use tokio::task::JoinHandle;

use self::awaited::{AwaitedRef, AwaitedRegistry, Predicate};
use self::serial_api_machine::{
    SerialApiMachine, SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineState,
};

pub use serial_api_machine::SerialApiMachineResult;

mod awaited;
mod interview_controller;
mod serial_api_machine;

submodule!(controller_commands);

type TaskCommandSender<T> = mpsc::Sender<T>;
type TaskCommandReceiver<T> = mpsc::Receiver<T>;

type SerialFrameEmitter = broadcast::Sender<SerialFrame>;
type SerialListener = broadcast::Receiver<SerialFrame>;

#[allow(dead_code)]
pub struct Driver {
    main_task: JoinHandle<()>,
    main_cmd: MainTaskCommandSender,
    main_task_shutdown: Arc<Notify>,

    serial_task: JoinHandle<()>,
    serial_cmd: SerialTaskCommandSender,
    serial_listener: SerialListener,
    serial_task_shutdown: Arc<Notify>,

    callback_id_gen: WrappingCounter<u8>,

    controller: Option<Controller>,
    state: DriverState,
}

#[derive(Default)]
struct DriverState {
    node_id_type: NodeIdType,
}

impl Driver {
    pub fn new(path: &str) -> Self {
        // The serial task owns the serial port. All communication needs to go through that task.
        let port = SerialPort::new(path).unwrap();
        // To control it, we send a thread command along with a "callback" oneshot channel to the task.
        let (serial_cmd_tx, serial_cmd_rx) = mpsc::channel::<SerialTaskCommand>(100);
        // The listener is used to receive frames from the serial port
        let (serial_listener_tx, serial_listener_rx) = broadcast::channel::<SerialFrame>(100);
        let serial_task_shutdown = Arc::new(Notify::new());
        let serial_task_shutdown2 = serial_task_shutdown.clone();

        // The main logic happens in another task that owns the internal state.
        // To control it, we need another channel.
        let (main_cmd_tx, main_cmd_rx) = mpsc::channel::<MainTaskCommand>(100);
        let main_serial_cmd = serial_cmd_tx.clone();
        let main_serial_listener = serial_listener_tx.subscribe();
        let main_task_shutdown = Arc::new(Notify::new());
        let main_task_shutdown2 = main_task_shutdown.clone();

        // Start the background task for the main logic
        let main_task = tokio::spawn(main_loop(
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

        let callback_id_gen = WrappingCounter::new();

        Self {
            main_task,
            main_cmd: main_cmd_tx,
            main_task_shutdown,
            serial_task,
            serial_cmd: serial_cmd_tx,
            serial_task_shutdown,
            serial_listener: serial_listener_rx,
            callback_id_gen,
            controller: None,
            state: DriverState::default(),
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        // Synchronize the serial port
        exec_background_task!(
            self.serial_cmd,
            SerialTaskCommand::SendFrame,
            SerialFrame::ControlFlow(ControlFlow::NAK)
        )?;

        // TODO: Interview controller
        self.controller = Some(self.interview_controller().await.unwrap());
        println!("Controller info: {:#?}", &self.controller);

        Ok(())
    }

    /// Write a frame to the serial port, returning a reference to the awaited ACK frame
    pub async fn write_serial(&self, frame: SerialFrame) -> Result<AwaitedRef<ControlFlow>> {
        // Register an awaiter for the ACK frame
        let ret = self
            .await_control_flow_frame(Box::new(|_| true), Some(Duration::from_millis(1600)))
            .await;
        // ...then send the frame
        exec_background_task!(&self.serial_cmd, SerialTaskCommand::SendFrame, frame)?;

        ret
    }

    async fn await_control_flow_frame(
        &self,
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>,
    ) -> Result<AwaitedRef<ControlFlow>> {
        // To await a control frame, we first register an awaiter
        exec_background_task!(
            self.main_cmd,
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
        // To await a command, we first register an awaiter
        exec_background_task!(
            self.main_cmd,
            MainTaskCommand::RegisterAwaitedCommand,
            predicate,
            timeout
        )
    }

    pub async fn execute_serial_api_command<C>(
        &mut self,
        mut command: C,
    ) -> Result<SerialApiMachineResult>
    where
        C: CommandRequest + Clone + 'static,
        SerialFrame: From<C>,
    {
        // Set up state machine and interpreter
        let mut state_machine = SerialApiMachine::new();

        // Give the command a callback ID if it needs one
        if command.needs_callback_id() {
            command.set_callback_id(Some(self.callback_id_gen.increment()));
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
                .await?,
            )
        };
        let mut awaited_callback: Option<AwaitedRef<Command>> = {
            let command = command.clone();
            Some(
                self.await_command(
                    Box::new(move |cmd| command.test_callback(cmd)),
                    Some(Duration::from_millis(30000)),
                )
                .await?,
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
                            // FIXME: We should take a reference here and use try_into()
                            let frame = SerialFrame::from(command.clone());
                            // Send the command to the controller
                            awaited_ack = Some(self.write_serial(frame).await?);
                            // and notify the state machine
                            next_input = Some(SerialApiMachineInput::FrameSent);
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
            serial_api_machine::SerialApiMachineState::Done(s) => Ok(s.clone()),
            _ => panic!(
                "Serial API machine finished with invalid state {:?}",
                final_state
            ),
        }
    }
}

macro_rules! define_task_commands {
    (
        $enum_name:ident$(<$($enum_lt:lifetime),+ $(,)?>)? {
            $( $cmd_name:ident$(<$($lt:lifetime),+ $(,)?>)? -> $cmd_result:ty {
                $( $field_name:ident : $field_type:ty ),* $(,)?
            } ),* $(,)?
        }
    ) => {
        enum $enum_name$(<$($enum_lt),+>)? {
            $(
                $cmd_name($cmd_name$(<$($lt),+>)?),
            )*
        }

        $(
            define_task_commands!(
                @variant $cmd_name$(<$($lt),+>)? -> $cmd_result {
                    $( $field_name : $field_type ),*
                }
            );
        )*
    };
    // Variant with lifetimes
    (
        @variant $cmd_name:ident<$($lt:lifetime),+ $(,)?> -> $cmd_result:ty {
            $( $field_name:ident : $field_type:ty ),* $(,)?
        }
    ) => {
        struct $cmd_name<$($lt),+> {
            $( pub $field_name: $field_type ),*,
            pub callback: oneshot::Sender<$cmd_result>,
        }

        impl<$($lt),+> $cmd_name<$($lt),+> {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> (Self, oneshot::Receiver<$cmd_result>) {
                let (tx, rx) = oneshot::channel::<$cmd_result>();
                (
                    Self {
                        $( $field_name ),*,
                        callback: tx,
                    },
                    rx,
                )
            }
        }
    };
    // Variant without lifetimes
    (
        @variant $cmd_name:ident -> $cmd_result:ty {
            $( $field_name:ident : $field_type:ty ),* $(,)?
        }
    ) => {
        struct $cmd_name {
            $( pub $field_name: $field_type ),*,
            pub callback: oneshot::Sender<$cmd_result>,
        }

        impl $cmd_name {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> (Self, oneshot::Receiver<$cmd_result>) {
                let (tx, rx) = oneshot::channel::<$cmd_result>();
                (
                    Self {
                        $( $field_name ),*,
                        callback: tx,
                    },
                    rx,
                )
            }
        }
    }
}

define_task_commands!(MainTaskCommand {
    RegisterAwaitedCommand -> AwaitedRef<Command> {
        predicate: Predicate<Command>,
        timeout: Option<Duration>
    },
    RegisterAwaitedControlFlowFrame -> AwaitedRef<ControlFlow> {
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>
    },
});

type MainTaskCommandSender = TaskCommandSender<MainTaskCommand>;
type MainTaskCommandReceiver = TaskCommandReceiver<MainTaskCommand>;

struct MainLoopState {
    awaited_control_flow_frames: Arc<AwaitedRegistry<ControlFlow>>,
    awaited_commands: Arc<AwaitedRegistry<Command>>,
}

async fn main_loop(
    mut cmd_rx: MainTaskCommandReceiver,
    shutdown: Arc<Notify>,
    serial_cmd: SerialTaskCommandSender,
    mut serial_listener: SerialListener,
    // command_handlers: Arc<Mutex<Vec<CommandHandler>>>,
) {
    let state = MainLoopState {
        awaited_control_flow_frames: Arc::new(AwaitedRegistry::default()),
        awaited_commands: Arc::new(AwaitedRegistry::default()),
    };

    loop {
        tokio::select! {
            // Make sure we don't read from the serial port if there is a potential command
            // waiting to set up a frame handler
            biased;

            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some(cmd) = cmd_rx.recv() => main_loop_handle_command(&state, cmd, &serial_cmd).await,

            // The serial port emitted a frame
            Ok(frame) = serial_listener.recv() => main_loop_handle_frame(&state, frame, &serial_cmd).await
        }
    }

    println!("main task stopped")
}

async fn main_loop_handle_command(
    state: &MainLoopState,
    cmd: MainTaskCommand,
    _serial_cmd: &SerialTaskCommandSender,
) {
    match cmd {
        MainTaskCommand::RegisterAwaitedControlFlowFrame(ctrl) => {
            let result = state
                .awaited_control_flow_frames
                .add(ctrl.predicate, ctrl.timeout);
            ctrl.callback
                .send(result)
                .map_err(|_| Error::Internal)
                .unwrap();
        }

        MainTaskCommand::RegisterAwaitedCommand(cmd) => {
            let result = state.awaited_commands.add(cmd.predicate, cmd.timeout);
            cmd.callback
                .send(result)
                .map_err(|_| Error::Internal)
                .unwrap();
        }

        #[allow(unreachable_patterns)]
        _ => {} // Ignore other commands
    }
}

async fn main_loop_handle_frame(
    state: &MainLoopState,
    frame: SerialFrame,
    _serial_cmd: &SerialTaskCommandSender,
) {
    // TODO: Consider if we need to always handle something here
    match &frame {
        SerialFrame::ControlFlow(cf) => {
            // If the awaited control-flow-frame registry has a matching awaiter,
            // remove it and send the frame through its channel
            if let Some(channel) = state.awaited_control_flow_frames.take_matching(cf) {
                channel.send(*cf).unwrap();

                #[allow(clippy::needless_return)]
                return;
            }
        }
        SerialFrame::Command(cmd) => {
            // If the awaited command registry has a matching awaiter,
            // remove it and send the command through its channel
            if let Some(channel) = state.awaited_commands.take_matching(cmd) {
                channel.send(cmd.clone()).unwrap();

                #[allow(clippy::needless_return)]
                return;
            }
        }
        _ => {}
    }
    // tokio::time::sleep(Duration::from_millis(10)).await;
}

define_task_commands!(SerialTaskCommand {
    // Send the given frame to the serial port
    SendFrame -> () {
        frame: SerialFrame
    },
    // Use the given node ID type for parsing frames
    UseNodeIDType -> () {
        node_id_type: NodeIdType
    }
});

type SerialTaskCommandSender = TaskCommandSender<SerialTaskCommand>;
type SerialTaskCommandReceiver = TaskCommandReceiver<SerialTaskCommand>;

#[derive(Default)]
struct SerialLoopState {
    node_id_type: NodeIdType,
}

async fn serial_loop(
    mut port: SerialPort,
    mut cmd_rx: SerialTaskCommandReceiver,
    shutdown: Arc<Notify>,
    frame_emitter: SerialFrameEmitter,
) {
    let mut state = SerialLoopState::default();
    loop {
        // Whatever happens first gets handled first.
        tokio::select! {
            // Make sure we don't read from the serial port if there is a command to be handled
            biased;

            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some(cmd) = cmd_rx.recv() => serial_loop_handle_command(&mut state, &mut port, cmd).await,

            // We received a frame from the serial port
            Some(frame) = port.read() => serial_loop_handle_frame(&state, &mut port, frame, &frame_emitter).await
        }
    }

    println!("serial task stopped")
}

async fn serial_loop_handle_command(
    state: &mut SerialLoopState,
    port: &mut SerialPort,
    cmd: SerialTaskCommand,
) {
    match cmd {
        SerialTaskCommand::SendFrame(SendFrame { frame, callback }) => {
            let ctx = CommandEncodingContext::builder()
                .node_id_type(state.node_id_type)
                .build()
                .unwrap();
            port.write(frame.try_into_raw(&ctx).unwrap()).await.unwrap();
            callback.send(()).unwrap();
        }
        SerialTaskCommand::UseNodeIDType(UseNodeIDType {
            node_id_type,
            callback,
        }) => {
            state.node_id_type = node_id_type;
            callback.send(()).unwrap();
        }
    }
}

async fn serial_loop_handle_frame(
    state: &SerialLoopState,
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
                    let ctx = CommandEncodingContext::builder()
                        .node_id_type(state.node_id_type)
                        .build()
                        .unwrap();
                    match zwave_serial::command::Command::try_from_raw(raw, &ctx) {
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

macro_rules! exec_background_task {
    ($command_sender:expr, $command_type:ident::$variant:ident, $($new_args:tt)*) => {
        {
            let (cmd, rx) = $variant::new($($new_args)*);
            let cmd = $command_type::$variant(cmd);
            $command_sender.send(cmd).await.map_err(|_| $crate::error::Error::Internal)?;
            rx.await.map_err(|_| $crate::error::Error::Internal)
        }
    };
}
pub(crate) use exec_background_task;

impl Drop for Driver {
    fn drop(&mut self) {
        // We need to stop the background tasks, otherwise they will stick around until the process exits
        self.serial_task_shutdown.notify_one();
        self.main_task_shutdown.notify_one();
    }
}
