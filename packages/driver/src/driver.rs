use self::storage::DriverStorage;
use crate::error::Result;
use driver_api::DriverApi;
use main_loop::{MainLoop, MainTaskCommand, MainTaskCommandSender};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread;
use tokio::sync::{broadcast, mpsc, Notify};
use tokio::task::JoinHandle;
use typed_builder::TypedBuilder;
use zwave_core::log::Loglevel;
use zwave_core::security::{SecurityManager, SecurityManagerOptions};
use zwave_core::util::now;
use zwave_core::{prelude::*, submodule};
use zwave_logging::loggers::{base::BaseLogger, driver::DriverLogger, serial::SerialLogger};
use zwave_logging::{Direction, LogInfo, Logger};
use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::{ControlFlow, RawSerialFrame, SerialFrame};
use zwave_serial::prelude::*;
use zwave_serial::serialport::{SerialPort, TcpSocket, ZWavePort};

mod awaited;
pub(crate) mod cache;
pub(crate) mod driver_api;
mod init_controller_and_nodes;
mod interview_nodes;
mod main_loop;
pub(crate) use main_loop::*;
mod serial_api_machine;
mod storage;

submodule!(driver_state);
submodule!(controller_commands);
submodule!(node_commands);
submodule!(node_api);
submodule!(background_logger);

type TaskCommandSender<T> = mpsc::Sender<T>;
type TaskCommandReceiver<T> = mpsc::Receiver<T>;

type SerialFrameEmitter = broadcast::Sender<SerialFrame>;
type SerialListener = broadcast::Receiver<SerialFrame>;

pub struct Driver<S: DriverState> {
    tasks: DriverTasks,
    options: DriverOptionsStatic,

    state: S,
    storage: Arc<DriverStorage>,

    /// The own API instance used by the driver
    api: DriverApi<S>,
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

    #[builder(default)]
    security_keys: SecurityKeys,
}

struct DriverOptionsStatic {
    pub loglevel: Loglevel,
    pub security_keys: SecurityKeys,
}

impl From<DriverOptions<'_>> for DriverOptionsStatic {
    fn from(options: DriverOptions) -> Self {
        Self {
            loglevel: options.loglevel,
            security_keys: options.security_keys,
        }
    }
}

#[derive(Default, Clone, TypedBuilder)]
pub struct SecurityKeys {
    #[builder(default, setter(into))]
    pub s0_legacy: Option<Vec<u8>>,
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
        let main_task_shutdown = Arc::new(Notify::new());
        let main_serial_listener = serial_listener_tx.subscribe();

        // Logging happens in a separate **thread** in order to not interfere with the main logic.
        let loglevel = options.loglevel; // FIXME: Add a way to change this at runtime
        let (log_cmd_tx, log_cmd_rx) = std::sync::mpsc::channel::<LogTaskCommand>();
        let bg_logger = Arc::new(BackgroundLogger::new(log_cmd_tx.clone(), loglevel));
        let serial_logger = SerialLogger::new(bg_logger.clone());
        let driver_logger = DriverLogger::new(bg_logger.clone());

        // Start the background thread for logging immediately, so we can log before opening the serial port
        let log_thread = thread::spawn(move || log_loop(log_cmd_rx, loglevel));

        driver_logger.logo();
        driver_logger.info(|| "version 0.0.1-alpha");
        driver_logger.info(|| "");
        driver_logger.info(|| format!("opening serial port {}", path));

        let open_port_result = if let Some(path) = path.strip_prefix("tcp://") {
            TcpSocket::new(path).map(ZWavePort::Tcp)
        } else {
            SerialPort::new(path).map(ZWavePort::Serial)
        };

        let port = match open_port_result {
            Ok(port) => {
                driver_logger.info(|| "serial port opened");
                port
            }
            Err(e) => {
                driver_logger.error(|| format!("failed to open serial port: {}", e));
                return Err(e.into());
            }
        };

        let driver_storage = Arc::new(DriverStorage::new(bg_logger, NodeIdType::NodeId8Bit));

        let state = Init;

        let api = DriverApi::new(
            state.clone(),
            main_cmd_tx.clone(),
            serial_cmd_tx.clone(),
            driver_storage.clone(),
        );

        // Start the background task for the main logic
        let mut main_loop = MainLoop::new(
            Box::new(api.clone()),
            main_task_shutdown.clone(),
            main_cmd_rx,
            main_serial_listener,
        );
        let main_task = tokio::spawn(async move {
            main_loop.run().await;
        });

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
            options: options.into(),
            storage: driver_storage,
            api,
        })
    }

    pub async fn init(self) -> Result<Driver<Ready>> {
        let logger = self.log();

        // Synchronize the serial port
        logger.verbose(|| "synchronizing serial port...");
        dispatch_async!(
            self.tasks.serial_cmd,
            SerialTaskCommand::SendFrame,
            RawSerialFrame::ControlFlow(ControlFlow::NAK)
        )??;

        let mut ready = self.interview_controller().await?;

        // Store our node ID so other tasks have access too
        self.storage.set_own_node_id(ready.controller.own_node_id);

        // Initialize security managers
        if let Some(s0_key) = &self.options.security_keys.s0_legacy {
            logger.info(|| "Network key for S0 configured, enabling S0 security manager...");
            let security_manager = SecurityManager::new(SecurityManagerOptions {
                own_node_id: ready.controller.own_node_id,
                network_key: s0_key.clone(),
            });
            ready.security_manager = Some(Arc::new(RwLock::new(security_manager)));
        } else {
            logger.warn(|| "No network key for S0 configured, communication with secure (S0) devices won't work!");
        }

        let ready_api = DriverApi::new(
            ready.clone(),
            self.tasks.main_cmd.clone(),
            self.tasks.serial_cmd.clone(),
            self.storage.clone(),
        );

        // Replace the main loop's driver API with the new one
        dispatch_async!(
            self.tasks.main_cmd,
            MainTaskCommand::SetDriverApi,
            Box::new(ready_api.clone())
        )?;

        let this = Driver::<Ready> {
            tasks: self.tasks,
            state: ready,
            options: self.options,
            storage: self.storage,
            api: ready_api,
        };

        this.configure_controller().await?;

        Ok(this)
    }
}

// TODO: Consider not exposing the entire API to the outside
impl<S> Deref for Driver<S>
where
    S: DriverState,
{
    type Target = DriverApi<S>;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

pub(crate) trait BackgroundTask {
    async fn run(&mut self);
}

macro_rules! define_async_task_commands {
    (
        $enum_name:ident$(<$($enum_lt:lifetime),+ $(,)?>)? {
            $( $cmd_name:ident$(<$($lt:lifetime),+ $(,)?>)? -> $cmd_result:ty {
                $( $field_name:ident : $field_type:ty ),* $(,)?
            } ),* $(,)?
        }
    ) => {
        pub(crate) enum $enum_name$(<$($enum_lt),+>)? {
            $(
                $cmd_name($cmd_name$(<$($lt),+>)?),
            )*
        }

        $(
            define_async_task_commands!(
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
        pub(crate) struct $cmd_name<$($lt),+> {
            // The callback of every async task must be used, otherwise the caller will panic
            #[forbid(dead_code)]
            pub callback: tokio::sync::oneshot::Sender<$cmd_result>,
            $( pub $field_name: $field_type ),*,
        }

        impl<$($lt),+> $cmd_name<$($lt),+> {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> (Self, tokio::sync::oneshot::Receiver<$cmd_result>) {
                let (tx, rx) = tokio::sync::oneshot::channel::<$cmd_result>();
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
        pub(crate) struct $cmd_name {
            // The callback of every async task must be used, otherwise the caller will panic
            #[forbid(dead_code)]
            pub callback: tokio::sync::oneshot::Sender<$cmd_result>,
            $( pub $field_name: $field_type ),*
        }

        impl $cmd_name {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> (Self, tokio::sync::oneshot::Receiver<$cmd_result>) {
                let (tx, rx) = tokio::sync::oneshot::channel::<$cmd_result>();
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
pub(crate) use define_async_task_commands;

macro_rules! define_oneshot_task_commands {
    (
        $enum_name:ident$(<$($enum_lt:lifetime),+ $(,)?>)? {
            $( $cmd_name:ident$(<$($lt:lifetime),+ $(,)?>)? {
                $( $field_name:ident : $field_type:ty ),* $(,)?
            } ),* $(,)?
        }
    ) => {
        pub(crate) enum $enum_name$(<$($enum_lt),+>)? {
            $(
                $cmd_name($cmd_name$(<$($lt),+>)?),
            )*
        }

        $(
            define_oneshot_task_commands!(
                @variant $cmd_name$(<$($lt),+>)? {
                    $( $field_name : $field_type ),*
                }
            );
        )*
    };
    // Variant with lifetimes
    (
        @variant $cmd_name:ident<$($lt:lifetime),+ $(,)?> {
            $( $field_name:ident : $field_type:ty ),* $(,)?
        }
    ) => {
        pub(crate) struct $cmd_name<$($lt),+> {
            $( pub $field_name: $field_type ),*,
        }

        impl<$($lt),+> $cmd_name<$($lt),+> {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> Self {
                Self {
                    $( $field_name ),*,
                }
            }
        }
    };
    // Variant without lifetimes
    (
        @variant $cmd_name:ident {
            $( $field_name:ident : $field_type:ty ),* $(,)?
        }
    ) => {
        pub(crate) struct $cmd_name {
            $( pub $field_name: $field_type ),*
        }

        impl $cmd_name {
            pub fn new(
                $( $field_name: $field_type ),*
            ) -> Self {
                Self {
                    $( $field_name ),*
                }
            }
        }
    }
}
pub(crate) use define_oneshot_task_commands;

define_async_task_commands!(SerialTaskCommand {
    // Send the given frame to the serial port
    SendFrame -> Result<()> {
        frame: RawSerialFrame
    },
});

type SerialTaskCommandSender = TaskCommandSender<SerialTaskCommand>;
type SerialTaskCommandReceiver = TaskCommandReceiver<SerialTaskCommand>;

struct SerialLoopStorage {
    logger: SerialLogger,
}

async fn serial_loop(
    mut port: ZWavePort,
    logger: SerialLogger,
    mut cmd_rx: SerialTaskCommandReceiver,
    shutdown: Arc<Notify>,
    frame_emitter: SerialFrameEmitter,
) {
    let mut storage = SerialLoopStorage { logger };

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
    port: &mut ZWavePort,
    cmd: SerialTaskCommand,
) {
    match cmd {
        SerialTaskCommand::SendFrame(SendFrame { frame, callback }) => {
            let result = write_serial(port, frame, &storage.logger).await;
            callback
                .send(result)
                .expect("invoking the callback of a SerialTaskCommand should not fail");
        }
    }
}

async fn serial_loop_handle_frame(
    storage: &SerialLoopStorage,
    port: &mut ZWavePort,
    frame: RawSerialFrame,
    frame_emitter: &SerialFrameEmitter,
) {
    let emit = match &frame {
        RawSerialFrame::Data(data) => {
            storage.logger.data(data, Direction::Inbound);
            // Try to parse the frame
            // TODO: Do we really need to clone the BytesMut here?
            match zwave_serial::command_raw::CommandRaw::parse(&mut data.clone()) {
                Ok(raw) => {
                    // The first step of parsing was successful, ACK the frame
                    write_serial(
                        port,
                        RawSerialFrame::ControlFlow(ControlFlow::ACK),
                        &storage.logger,
                    )
                    .await
                    .unwrap();

                    Some(SerialFrame::Command(raw))
                }
                Err(e) => {
                    println!("{} error: {}", now(), e);
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
    port: &mut ZWavePort,
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

define_oneshot_task_commands!(LogTaskCommand {
    // Set the log level of the given logger
    UseLogLevel {
        level: Loglevel,
    },
    // Log the given message
    Log {
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
        LogTaskCommand::UseLogLevel(UseLogLevel { level }) => {
            storage.logger.set_log_level(level);
        }

        LogTaskCommand::Log(Log { log, level }) => {
            storage.logger.log(log, level);
        }

        // Ignore other commands
        _ => {}
    }
}

/// Execute the given command in the given background task and await the result.
/// ```ignore
/// dispatch_async!(
///     task_ref: &Sender<TaskCommand>,
///     TaskCommand::Variant,
///     ...args
/// )?
/// ```
///
/// The command enum MUST be generated with the `define_async_task_commands!` macro.
/// The second argument to the macro is the variant of the command enum to execute, but without arguments, if there are any.
/// The arguments of the command are passed to the macro as the remaining arguments.
///
/// This invocation will return the result of the command execution, or an `Error::Internal`, if there was a problem communicating
/// with the background task. To convey an error, the task must return a `Result` itself.
macro_rules! dispatch_async {
    ($command_sender:expr, $command_type:ident::$variant:ident, $($new_args:tt)*) => {
        {
            let (cmd, rx) = $crate::driver::$variant::new($($new_args)*);
            let cmd = $crate::driver::$command_type::$variant(cmd);
            $command_sender.send(cmd).await.map_err(|_| $crate::error::Error::Internal)?;
            rx.await.map_err(|_| $crate::error::Error::Internal)
        }
    };

    ($command_sender:expr, $command_type:ident::$variant:ident) => {
        dispatch_async!($command_sender, $command_type::$variant,)
    }

}
pub(crate) use dispatch_async;

/// Execute the given command in the given background task **without** waiting for the result.
/// ```ignore
/// dispatch_oneshot!(
///     task_ref: &Sender<TaskCommand>,
///     TaskCommand::Variant,
///     ...args
/// )?
/// ```
///
/// The command enum MUST be generated with the `define_oneshot_task_commands!` macro.
/// The second argument to the macro is the variant of the command enum to execute, but without arguments, if there are any.
/// The arguments of the command are passed to the macro as the remaining arguments.
///
/// This invocation will return `()`, or an `Error::Internal`, if there was a problem communicating
/// with the background task.
macro_rules! dispatch_oneshot {
    ($command_sender:expr, $command_type:ident::$variant:ident, $($new_args:tt)*) => {
        {
            let cmd = $crate::driver::$variant::new($($new_args)*);
            let cmd = $crate::driver::$command_type::$variant(cmd);
            $command_sender.send(cmd).map_err(|_| $crate::error::Error::Internal)
        }
    };

    ($command_sender:expr, $command_type:ident::$variant:ident) => {
        dispatch_oneshot!($command_sender, $command_type::$variant,)
    }

}
pub(crate) use dispatch_oneshot;
