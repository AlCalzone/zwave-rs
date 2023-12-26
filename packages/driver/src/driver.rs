use std::sync::Arc;
use std::time::Duration;

use zwave_cc::commandclass::{WithAddress, CC};
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::util::now;
use zwave_core::wrapping_counter::WrappingCounter;
use zwave_core::{prelude::*, submodule};
use zwave_serial::prelude::*;

use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::{ControlFlow, RawSerialFrame, SerialFrame};
use zwave_serial::serialport::SerialPort;

use crate::error::{Error, Result};

use tokio::sync::{broadcast, mpsc, oneshot, Notify};
use tokio::task::JoinHandle;

use self::awaited::{AwaitedRef, AwaitedRegistry, Predicate};
use self::serial_api_machine::{
    SerialApiMachine, SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineState,
};
use self::storage::DriverStorage;

pub use serial_api_machine::SerialApiMachineResult;

mod awaited;
mod interview_controller;
mod interview_nodes;
mod serial_api_machine;
mod storage;

submodule!(driver_state);
submodule!(controller_commands);
submodule!(node_commands);
submodule!(node_api);
submodule!(controller_api);

type TaskCommandSender<T> = mpsc::Sender<T>;
type TaskCommandReceiver<T> = mpsc::Receiver<T>;

type SerialFrameEmitter = broadcast::Sender<SerialFrame>;
type SerialListener = broadcast::Receiver<SerialFrame>;

pub struct Driver<S: DriverState> {
    tasks: DriverTasks,

    state: S,
    storage: DriverStorage,
}

#[allow(dead_code)]
struct DriverTasks {
    main_task: JoinHandle<()>,
    main_cmd: MainTaskCommandSender,
    main_task_shutdown: Arc<Notify>,

    serial_task: JoinHandle<()>,
    serial_cmd: SerialTaskCommandSender,
    serial_listener: SerialListener,
    serial_task_shutdown: Arc<Notify>,
}

impl Drop for DriverTasks {
    fn drop(&mut self) {
        // We need to stop the background tasks, otherwise they will stick around until the process exits
        self.serial_task_shutdown.notify_one();
        self.main_task_shutdown.notify_one();
    }
}

impl Driver<Init> {
    pub fn new(path: &str) -> Result<Self> {
        // The serial task owns the serial port. All communication needs to go through that task.
        let port = SerialPort::new(path)?;

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

        let tasks = DriverTasks {
            main_task,
            main_cmd: main_cmd_tx,
            main_task_shutdown,
            serial_task,
            serial_cmd: serial_cmd_tx,
            serial_task_shutdown,
            serial_listener: serial_listener_rx,
        };

        Ok(Self {
            tasks,
            state: Init,
            storage: DriverStorage::default(),
        })
    }

    pub async fn init(self) -> Result<Driver<Ready>> {
        // Synchronize the serial port
        exec_background_task!(
            self.tasks.serial_cmd,
            SerialTaskCommand::SendFrame,
            SerialFrame::ControlFlow(ControlFlow::NAK)
        )??;

        let ready = self.interview_controller().await?;
        println!("Controller info: {:#?}", &ready);

        let mut this = Driver::<Ready> {
            tasks: self.tasks,
            state: ready,
            storage: self.storage,
        };

        this.configure_controller().await?;

        Ok(this)
    }
}

impl<S> Driver<S>
where
    S: DriverState,
{
    /// Write a frame to the serial port, returning a reference to the awaited ACK frame
    pub async fn write_serial(&self, frame: SerialFrame) -> Result<AwaitedRef<ControlFlow>> {
        // Register an awaiter for the ACK frame
        let ret = self
            .await_control_flow_frame(Box::new(|_| true), Some(Duration::from_millis(1600)))
            .await;
        // ...then send the frame
        exec_background_task!(&self.tasks.serial_cmd, SerialTaskCommand::SendFrame, frame)??;

        ret
    }

    async fn await_control_flow_frame(
        &self,
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>,
    ) -> Result<AwaitedRef<ControlFlow>> {
        exec_background_task!(
            self.tasks.main_cmd,
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
        exec_background_task!(
            self.tasks.main_cmd,
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
        exec_background_task!(
            self.tasks.main_cmd,
            MainTaskCommand::RegisterAwaitedCC,
            predicate,
            timeout
        )
    }

    pub async fn get_next_callback_id(&self) -> Result<u8> {
        exec_background_task!(self.tasks.main_cmd, MainTaskCommand::GetNextCallbackId)
    }

    pub async fn execute_serial_api_command<C>(
        &self,
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
            pub callback: oneshot::Sender<$cmd_result>,
            $( pub $field_name: $field_type ),*
        }

        impl $cmd_name {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> (Self, oneshot::Receiver<$cmd_result>) {
                let (tx, rx) = oneshot::channel::<$cmd_result>();
                (
                    Self {
                        callback: tx,
                        $( $field_name ),*
                    },
                    rx,
                )
            }
        }
    }
}

define_task_commands!(MainTaskCommand {
    RegisterAwaitedCC -> AwaitedRef<WithAddress<CC>> {
        predicate: Predicate<WithAddress<CC>>,
        timeout: Option<Duration>
    },
    RegisterAwaitedCommand -> AwaitedRef<Command> {
        predicate: Predicate<Command>,
        timeout: Option<Duration>
    },
    RegisterAwaitedControlFlowFrame -> AwaitedRef<ControlFlow> {
        predicate: Predicate<ControlFlow>,
        timeout: Option<Duration>
    },
    GetNextCallbackId -> u8 {}
});

type MainTaskCommandSender = TaskCommandSender<MainTaskCommand>;
type MainTaskCommandReceiver = TaskCommandReceiver<MainTaskCommand>;

struct MainLoopStorage {
    awaited_control_flow_frames: Arc<AwaitedRegistry<ControlFlow>>,
    awaited_commands: Arc<AwaitedRegistry<Command>>,
    awaited_ccs: Arc<AwaitedRegistry<WithAddress<CC>>>,
    callback_id_gen: WrappingCounter<u8>,
}

async fn main_loop(
    mut cmd_rx: MainTaskCommandReceiver,
    shutdown: Arc<Notify>,
    serial_cmd: SerialTaskCommandSender,
    mut serial_listener: SerialListener,
    // command_handlers: Arc<Mutex<Vec<CommandHandler>>>,
) {
    let mut storage = MainLoopStorage {
        awaited_control_flow_frames: Arc::new(AwaitedRegistry::default()),
        awaited_commands: Arc::new(AwaitedRegistry::default()),
        awaited_ccs: Arc::new(AwaitedRegistry::default()),
        callback_id_gen: WrappingCounter::new(),
    };

    loop {
        tokio::select! {
            // Make sure we don't read from the serial port if there is a potential command
            // waiting to set up a frame handler
            biased;

            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some(cmd) = cmd_rx.recv() => main_loop_handle_command(&mut storage, cmd, &serial_cmd).await,

            // The serial port emitted a frame
            Ok(frame) = serial_listener.recv() => main_loop_handle_frame(&storage, frame, &serial_cmd).await
        }
    }

    println!("main task stopped")
}

async fn main_loop_handle_command(
    storage: &mut MainLoopStorage,
    cmd: MainTaskCommand,
    _serial_cmd: &SerialTaskCommandSender,
) {
    match cmd {
        MainTaskCommand::RegisterAwaitedControlFlowFrame(ctrl) => {
            let result = storage
                .awaited_control_flow_frames
                .add(ctrl.predicate, ctrl.timeout);
            ctrl.callback
                .send(result)
                .expect("invoking the callback of a MainTaskCommand should not fail");
        }

        MainTaskCommand::RegisterAwaitedCommand(cmd) => {
            let result = storage.awaited_commands.add(cmd.predicate, cmd.timeout);
            cmd.callback
                .send(result)
                .expect("invoking the callback of a MainTaskCommand should not fail");
        }

        MainTaskCommand::RegisterAwaitedCC(cc) => {
            let result = storage.awaited_ccs.add(cc.predicate, cc.timeout);
            cc.callback
                .send(result)
                .expect("invoking the callback of a MainTaskCommand should not fail");
        }

        MainTaskCommand::GetNextCallbackId(cmd) => {
            let id = storage.callback_id_gen.increment();
            cmd.callback
                .send(id)
                .expect("invoking the callback of a MainTaskCommand should not fail");
        }
    }
}

async fn main_loop_handle_frame(
    storage: &MainLoopStorage,
    frame: SerialFrame,
    _serial_cmd: &SerialTaskCommandSender,
) {
    // TODO: Consider if we need to always handle something here
    match &frame {
        SerialFrame::ControlFlow(cf) => {
            // If the awaited control-flow-frame registry has a matching awaiter,
            // remove it and send the frame through its channel
            if let Some(channel) = storage.awaited_control_flow_frames.take_matching(cf) {
                channel
                    .send(*cf)
                    .expect("invoking the callback of an Awaited should not fail");

                #[allow(clippy::needless_return)]
                return;
            }
        }
        SerialFrame::Command(cmd) => {
            // If the awaited command registry has a matching awaiter,
            // remove it and send the command through its channel
            if let Some(channel) = storage.awaited_commands.take_matching(cmd) {
                channel
                    .send(cmd.clone())
                    .expect("invoking the callback of an Awaited should not fail");

                #[allow(clippy::needless_return)]
                return;
            }

            // Otherwise, figure out what to do with the command
            // TODO: This is a bit awkward due to the duplication
            match cmd {
                Command::ApplicationCommandRequest(cmd) => {
                    // If the awaited CC registry has a matching awaiter,
                    // remove it and send the CC through its channel
                    if let Some(channel) = storage.awaited_ccs.take_matching(&cmd.command) {
                        channel
                            .send(cmd.command.clone())
                            .expect("invoking the callback of an Awaited should not fail");

                        return;
                    }
                }
                Command::BridgeApplicationCommandRequest(cmd) => {
                    // If the awaited CC registry has a matching awaiter,
                    // remove it and send the CC through its channel
                    if let Some(channel) = storage.awaited_ccs.take_matching(&cmd.command) {
                        channel
                            .send(cmd.command.clone())
                            .expect("invoking the callback of an Awaited should not fail");

                        return;
                    }
                }
                _ => {}
            }

            println!("TODO: Handle command {:?}", cmd);
        }
        _ => {}
    }
    // tokio::time::sleep(Duration::from_millis(10)).await;
}

define_task_commands!(SerialTaskCommand {
    // Send the given frame to the serial port
    SendFrame -> Result<()> {
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
struct SerialLoopStorage {
    node_id_type: NodeIdType,
}

async fn serial_loop(
    mut port: SerialPort,
    mut cmd_rx: SerialTaskCommandReceiver,
    shutdown: Arc<Notify>,
    frame_emitter: SerialFrameEmitter,
) {
    let mut storage = SerialLoopStorage::default();
    loop {
        // Whatever happens first gets handled first.
        tokio::select! {
            // Make sure we don't read from the serial port if there is a command to be handled
            biased;

            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some(cmd) = cmd_rx.recv() => serial_loop_handle_command(&mut storage, &mut port, cmd).await,

            // We received a frame from the serial port
            Some(frame) = port.read() => serial_loop_handle_frame(&storage, &mut port, frame, &frame_emitter).await
        }
    }

    println!("serial task stopped")
}

async fn serial_loop_handle_command(
    storage: &mut SerialLoopStorage,
    port: &mut SerialPort,
    cmd: SerialTaskCommand,
) {
    match cmd {
        SerialTaskCommand::SendFrame(SendFrame { frame, callback }) => {
            let ctx = CommandEncodingContext::builder()
                .node_id_type(storage.node_id_type)
                .build();

            // Try encoding the frame // TODO: Expose encoding errors
            let result = frame.try_into_raw(&ctx).map_err(|_| Error::Internal);

            let result = if let Ok(raw) = result {
                port.write(raw).await.map_err(|e| e.into())
            } else {
                result.map(|_| ())
            };

            callback
                .send(result)
                .expect("invoking the callback of a SerialTaskCommand should not fail");
        }
        SerialTaskCommand::UseNodeIDType(UseNodeIDType {
            node_id_type,
            callback,
        }) => {
            storage.node_id_type = node_id_type;
            callback
                .send(())
                .expect("invoking the callback of a SerialTaskCommand should not fail");
        }
    }
}

async fn serial_loop_handle_frame(
    storage: &SerialLoopStorage,
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
                        .node_id_type(storage.node_id_type)
                        .build();
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
        let _ = frame_emitter.send(frame);
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

    ($command_sender:expr, $command_type:ident::$variant:ident) => {
        exec_background_task!($command_sender, $command_type::$variant,)
    }

}
pub(crate) use exec_background_task;
