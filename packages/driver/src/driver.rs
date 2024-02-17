use self::awaited::{AwaitedRef, AwaitedRegistry, Predicate};
use self::cache::ValueCache;
use self::serial_api_machine::{
    SerialApiMachine, SerialApiMachineCondition, SerialApiMachineInput, SerialApiMachineState,
};
use self::storage::{DriverStorage, DriverStorageShared};
use crate::error::{Error, Result};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot, Notify};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use zwave_cc::commandclass::{CCValues, WithAddress, CC};
use zwave_core::cache::Cache;
use zwave_core::log::Loglevel;
use zwave_core::state_machine::{StateMachine, StateMachineTransition};
use zwave_core::util::now;
use zwave_core::value_id::EndpointValueId;
use zwave_core::wrapping_counter::WrappingCounter;
use zwave_core::{prelude::*, submodule};
use zwave_logging::loggers::base::BaseLogger;
use zwave_logging::loggers::controller::ControllerLogger;
use zwave_logging::loggers::driver::DriverLogger;
use zwave_logging::loggers::node::NodeLogger;
use zwave_logging::loggers::serial::SerialLogger;
use zwave_logging::{Direction, LogInfo, Logger};
use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::{ControlFlow, RawSerialFrame, SerialFrame};
use zwave_serial::prelude::*;
use zwave_serial::serialport::SerialPort;

pub use serial_api_machine::SerialApiMachineResult;

mod awaited;
pub(crate) mod cache;
mod init_controller_and_nodes;
mod interview_nodes;
mod serial_api_machine;
mod storage;

submodule!(driver_state);
submodule!(controller_commands);
submodule!(node_commands);
submodule!(node_api);
submodule!(controller_api);
submodule!(background_logger);

type TaskCommandSender<T> = mpsc::Sender<T>;
type TaskCommandReceiver<T> = mpsc::Receiver<T>;

type SerialFrameEmitter = broadcast::Sender<SerialFrame>;
type SerialListener = broadcast::Receiver<SerialFrame>;

pub struct Driver<S: DriverState> {
    tasks: DriverTasks,

    state: S,
    storage: DriverStorage,
    shared_storage: Arc<DriverStorageShared>,
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

    log_thread: std::thread::JoinHandle<()>,
    log_cmd: LogTaskCommandSender,
}

impl Drop for DriverTasks {
    fn drop(&mut self) {
        // We need to stop the background tasks, otherwise they will stick around until the process exits
        self.serial_task_shutdown.notify_one();
        self.main_task_shutdown.notify_one();
        // The thread(s) will exit when the channel is closed
    }
}

#[derive(TypedBuilder)]
pub struct DriverOptions<'a> {
    path: &'a str,
    #[builder(default = Loglevel::Debug)]
    loglevel: Loglevel,
}

impl Driver<Init> {
    pub fn new(options: DriverOptions) -> Result<Self> {
        // The serial task owns the serial port. All communication needs to go through that task.
        let path = options.path;

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

        // Logging happens in a separate **thread** in order to not interfere with the main logic.
        let loglevel = options.loglevel; // FIXME: Add a way to change this at runtime
        let (log_cmd_tx, log_cmd_rx) = std::sync::mpsc::channel::<LogTaskCommand>();
        let bg_logger = Arc::new(BackgroundLogger::new(log_cmd_tx.clone(), loglevel));
        let serial_logger = SerialLogger::new(bg_logger.clone());
        let driver_logger = DriverLogger::new(bg_logger.clone());
        let controller_logger = ControllerLogger::new(bg_logger.clone());

        // Start the background thread for logging immediately, so we can log before opening the serial port
        let log_thread = thread::spawn(move || log_loop(log_cmd_rx, loglevel));

        driver_logger.logo();
        driver_logger.info(|| "version 0.0.1-alpha");
        driver_logger.info(|| "");
        driver_logger.info(|| format!("opening serial port {}", path));

        let port = match SerialPort::new(path) {
            Ok(port) => {
                driver_logger.info(|| "serial port opened");
                port
            }
            Err(e) => {
                driver_logger.error(|| format!("failed to open serial port: {}", e));
                return Err(e.into());
            }
        };

        let storage = DriverStorage::new(Default::default(), driver_logger, controller_logger);
        let shared_storage = Arc::new(DriverStorageShared::new(bg_logger));

        // Start the background task for the main logic
        let main_task = tokio::spawn(main_loop(
            main_cmd_rx,
            main_task_shutdown2,
            main_serial_cmd,
            main_serial_listener,
            shared_storage.clone(),
        ));

        // Start the background task for the serial port communication
        let serial_task = tokio::spawn(serial_loop(
            port,
            serial_logger,
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
            log_cmd: log_cmd_tx,
            log_thread,
        };

        Ok(Self {
            tasks,
            state: Init,
            storage,
            shared_storage,
        })
    }

    pub async fn init(self) -> Result<Driver<Ready>> {
        let logger = self.log();

        // Synchronize the serial port
        logger.verbose(|| "synchronizing serial port...");
        exec_background_task!(
            self.tasks.serial_cmd,
            SerialTaskCommand::SendFrame,
            SerialFrame::ControlFlow(ControlFlow::NAK)
        )??;

        let ready = self.interview_controller().await?;

        let mut this = Driver::<Ready> {
            tasks: self.tasks,
            state: ready,
            storage: self.storage,
            shared_storage: self.shared_storage,
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
        exec_background_task!(
            &self.tasks.serial_cmd,
            SerialTaskCommand::SendFrame,
            frame.clone()
        )??;

        // And log the command information if this was a command
        if let SerialFrame::Command(cmd) = &frame {
            let node_id = match cmd {
                // FIXME: Extract the endpoint index aswell
                Command::SendDataRequest(cmd) => Some(cmd.node_id),
                _ => None,
            };

            if let Some(node_id) = node_id {
                self.node_log(node_id, EndpointIndex::Root)
                    .command(cmd, Direction::Outbound);
            } else {
                self.storage
                    .controller_logger()
                    .command(cmd, Direction::Outbound);
            }
        }

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

    pub fn log(&self) -> &DriverLogger {
        self.storage.driver_logger()
    }

    pub fn controller_log(&self) -> &ControllerLogger {
        self.storage.controller_logger()
    }

    pub fn node_log(&self, node_id: NodeId, endpoint: EndpointIndex) -> NodeLogger {
        NodeLogger::new(self.shared_storage.logger().clone(), node_id, endpoint)
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
    bg_logger: Arc<BackgroundLogger>,
    driver_logger: DriverLogger,
    controller_logger: ControllerLogger,
}

async fn main_loop(
    mut cmd_rx: MainTaskCommandReceiver,
    shutdown: Arc<Notify>,
    serial_cmd: SerialTaskCommandSender,
    mut serial_listener: SerialListener,
    shared_storage: Arc<DriverStorageShared>,
    // command_handlers: Arc<Mutex<Vec<CommandHandler>>>,
) {
    let bg_logger = shared_storage.logger().clone();
    let driver_logger = DriverLogger::new(bg_logger.clone());
    let controller_logger = ControllerLogger::new(bg_logger.clone());

    let mut storage = MainLoopStorage {
        awaited_control_flow_frames: Arc::new(AwaitedRegistry::default()),
        awaited_commands: Arc::new(AwaitedRegistry::default()),
        awaited_ccs: Arc::new(AwaitedRegistry::default()),
        callback_id_gen: WrappingCounter::new(),
        bg_logger,
        driver_logger,
        controller_logger,
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
            Ok(frame) = serial_listener.recv() => main_loop_handle_frame(&storage, frame, &serial_cmd, &shared_storage).await
        }
    }
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

fn persist_cc_values(cc: &WithAddress<CC>, cache: &mut ValueCache) {
    // FIXME: Recurse through encapsulation CCs
    cache.write_many(cc.to_values().into_iter().map(|(value_id, value)| {
        let value_id = EndpointValueId::new(
            cc.address().source_node_id,
            cc.address().endpoint_index,
            value_id,
        );
        println!("Persisting {:?} = {:?}", value_id, value);
        (value_id, value)
    }));
}

async fn main_loop_handle_frame(
    storage: &MainLoopStorage,
    frame: SerialFrame,
    _serial_cmd: &SerialTaskCommandSender,
    shared_storage: &Arc<DriverStorageShared>,
) {
    // Persist CC values. TODO: test first if we should
    if let SerialFrame::Command(cmd) = &frame {
        let cc = match cmd {
            Command::ApplicationCommandRequest(cmd) => Some(&cmd.command),
            Command::BridgeApplicationCommandRequest(cmd) => Some(&cmd.command),
            _ => None,
        };
        if let Some(cc) = cc {
            let mut cache = ValueCache::new(shared_storage);
            persist_cc_values(cc, &mut cache);
        }
    }

    // Log the received command
    if let SerialFrame::Command(cmd) = &frame {
        let address = match cmd {
            Command::ApplicationCommandRequest(cmd) => Some(cmd.command.address()),
            Command::BridgeApplicationCommandRequest(cmd) => Some(cmd.command.address()),
            _ => None,
        };

        if let Some(address) = address {
            let node_logger = NodeLogger::new(
                storage.bg_logger.clone(),
                address.source_node_id,
                address.endpoint_index,
            );
            node_logger.command(cmd, Direction::Inbound);
        } else {
            storage.controller_logger.command(cmd, Direction::Inbound);
        }
    }

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

struct SerialLoopStorage {
    node_id_type: NodeIdType,
    logger: SerialLogger,
}

async fn serial_loop(
    mut port: SerialPort,
    logger: SerialLogger,
    mut cmd_rx: SerialTaskCommandReceiver,
    shutdown: Arc<Notify>,
    frame_emitter: SerialFrameEmitter,
) {
    let mut storage = SerialLoopStorage {
        node_id_type: Default::default(),
        logger,
    };

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
                write_serial(port, raw, &storage.logger).await
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
            storage.logger.data(data, Direction::Inbound);
            // Try to parse the frame
            match zwave_serial::command_raw::CommandRaw::parse(data) {
                Ok((_, raw)) => {
                    // The first step of parsing was successful, ACK the frame
                    write_serial(
                        port,
                        RawSerialFrame::ControlFlow(ControlFlow::ACK),
                        &storage.logger,
                    )
                    .await
                    .unwrap();

                    // Now try to convert it into an actual command
                    let ctx = CommandEncodingContext::builder()
                        .node_id_type(storage.node_id_type)
                        .build();
                    match zwave_serial::command::Command::try_from_raw(raw, &ctx) {
                        Ok(cmd) => Some(SerialFrame::Command(cmd)),
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
                    write_serial(
                        port,
                        RawSerialFrame::ControlFlow(ControlFlow::NAK),
                        &storage.logger,
                    )
                    .await
                    .unwrap();
                    None
                }
            }
        }
        RawSerialFrame::Garbage(data) => {
            storage.logger.discarded(data);
            // After receiving garbage, try to re-sync by sending NAK
            write_serial(
                port,
                RawSerialFrame::ControlFlow(ControlFlow::NAK),
                &storage.logger,
            )
            .await
            .unwrap();
            None
        }
        RawSerialFrame::ControlFlow(byte) => {
            storage.logger.control_flow(byte, Direction::Inbound);
            Some(SerialFrame::ControlFlow(*byte))
        }
    };

    if let Some(frame) = emit {
        let _ = frame_emitter.send(frame);
    }
}

async fn write_serial(
    port: &mut SerialPort,
    frame: RawSerialFrame,
    logger: &SerialLogger,
) -> Result<()> {
    match &frame {
        RawSerialFrame::Data(data) => {
            logger.data(data, Direction::Outbound);
        }
        RawSerialFrame::ControlFlow(byte) => {
            logger.control_flow(byte, Direction::Outbound);
        }
        _ => {}
    }

    port.write(frame).await.map_err(|e| e.into())
}

// FIXME: We need a variant for threads
define_task_commands!(LogTaskCommand {
    // Set the log level of the given logger
    UseLogLevel -> () {
        level: Loglevel,
    },
    // Log the given message
    Log -> () {
        log: LogInfo,
        level: Loglevel,
    },
});

type LogTaskCommandSender = std::sync::mpsc::Sender<LogTaskCommand>;
type LogTaskCommandReceiver = std::sync::mpsc::Receiver<LogTaskCommand>;

struct LogLoopStorage {
    logger: Box<dyn Logger>,
}

fn log_loop(cmd_rx: LogTaskCommandReceiver, loglevel: Loglevel) {
    let logger = BaseLogger {
        level: loglevel,
        writer: Box::new(termcolor::StandardStream::stdout(
            termcolor::ColorChoice::Auto,
        )),
        formatter: Box::new(zwave_logging::formatters::DefaultFormatter::new()),
    };

    let mut storage = LogLoopStorage {
        logger: Box::new(logger),
    };
    while let Ok(cmd) = cmd_rx.recv() {
        log_loop_handle_command(&mut storage, cmd);
    }
}

fn log_loop_handle_command(storage: &mut LogLoopStorage, cmd: LogTaskCommand) {
    match cmd {
        LogTaskCommand::UseLogLevel(UseLogLevel { level, callback: _ }) => {
            storage.logger.set_log_level(level);
        }

        LogTaskCommand::Log(Log {
            callback: _,
            log,
            level,
        }) => {
            storage.logger.log(log, level);
        }

        // Ignore other commands
        _ => {}
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

// FIXME: This is a shitty name
macro_rules! exec_background_task2 {
    ($command_sender:expr, $command_type:ident::$variant:ident, $($new_args:tt)*) => {
        {
            let (cmd, _rx) = $variant::new($($new_args)*);
            let cmd = $command_type::$variant(cmd);
            $command_sender.send(cmd).map_err(|_| $crate::error::Error::Internal)
        }
    };

    ($command_sender:expr, $command_type:ident::$variant:ident) => {
        exec_background_task2!($command_sender, $command_type::$variant,)
    }

}
pub(crate) use exec_background_task2;
