use std::sync::Arc;
use std::time::Duration;

use zwave_core::prelude::*;
use zwave_core::util::MaybeSleep;
use zwave_serial::prelude::*;

use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::{RawSerialFrame, SerialFrame};
use zwave_serial::serialport::SerialPort;

use crate::error::{Error, Result};

use tokio::sync::{broadcast, mpsc, oneshot, Mutex, Notify};
use tokio::task::JoinHandle;

use self::awaited::{AwaitedRegistry, Predicate};

mod awaited;
mod serial_api_machine;

enum ThreadCommand {
    Send(SerialFrame),
}

type ThreadCommandSender = mpsc::Sender<(ThreadCommand, oneshot::Sender<()>)>;
type ThreadCommandReceiver = mpsc::Receiver<(ThreadCommand, oneshot::Sender<()>)>;
type SerialFrameEmitter = broadcast::Sender<SerialFrame>;
type SerialListener = broadcast::Receiver<SerialFrame>;

type CommandHandler = Box<dyn Fn(Command) -> bool + Send + Sync>;

struct DriverState {
    command_handlers: Mutex<Vec<CommandHandler>>,
    awaited_commands: AwaitedRegistry<Command>,
}

#[allow(dead_code)]
pub struct Driver {
    this: Arc<DriverState>,

    serial_task: JoinHandle<()>,
    main_task: JoinHandle<()>,
    main_cmd: ThreadCommandSender,
    main_task_shutdown: Arc<Notify>,
    serial_cmd: ThreadCommandSender,
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
            mpsc::channel::<(ThreadCommand, oneshot::Sender<()>)>(100);
        // The listener is used to receive frames from the serial port
        let (serial_listener_tx, serial_listener_rx) = broadcast::channel::<SerialFrame>(100);
        let serial_task_shutdown = Arc::new(Notify::new());
        let serial_task_shutdown2 = serial_task_shutdown.clone();

        // The main logic happens in another task that owns the internal state.
        // To control it, we need another channel.
        let (main_cmd_tx, main_cmd_rx) = mpsc::channel::<(ThreadCommand, oneshot::Sender<()>)>(100);
        let main_serial_cmd = serial_cmd_tx.clone();
        let main_serial_listener = serial_listener_tx.subscribe();
        let main_task_shutdown = Arc::new(Notify::new());
        let main_task_shutdown2 = main_task_shutdown.clone();

        // let command_handlers: Vec<SerialCommandHandlerSender> = Vec::new();
        // let command_handlers = Arc::new(Mutex::new(command_handlers));
        let command_handlers: Vec<CommandHandler> = Vec::new();
        let command_handlers = Mutex::new(command_handlers);

        let awaited_commands = AwaitedRegistry::default();

        let this = DriverState {
            command_handlers,
            awaited_commands,
        };
        let this = Arc::new(this);

        // Start the background task for the main logic
        let main_task = tokio::spawn(main_loop(
            this.clone(),
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
            this,
        }
    }

    pub async fn write_serial(&self, frame: SerialFrame) -> Result<()> {
        send_thread_command(&self.serial_cmd, ThreadCommand::Send(frame)).await
    }

    pub async fn register_command_handler(&mut self, handler: CommandHandler) {
        let mut handlers = self.this.command_handlers.lock().await;
        handlers.push(handler);
        println!("registered command handler, count: {}", handlers.len());
    }

    pub async fn await_command(
        &self,
        predicate: Predicate<Command>,
        timeout: Option<Duration>,
    ) -> Option<Command> {
        // To await a command, we first register an awaiter
        let mut awaiter = self.this.awaited_commands.add(predicate); // self.register_command_awaiter(predicate);

        // ...wait for it to be fulfilled or time out
        let sleep = MaybeSleep::new(timeout);
        tokio::select! {
            cmd = awaiter.take_channel() => Some(cmd.unwrap()),
            _ = sleep => None,
        }
    }

    // pub async fn unregister_command_handler(&mut self, handler: CommandHandler) {
    //     self.command_handlers.retain(|h| h != &handler);
    // }

    // pub async fn register_command_handler(&mut self) -> SerialCommandHandlerReceiver {
    //     let id = 1;
    //     let (tx, rx) = mpsc::channel::<(Command, oneshot::Sender<bool>)>(100);
    //     let sender = SerialCommandHandlerSender { id, channel: tx };
    //     let receiver = SerialCommandHandlerReceiver::new(id, rx);
    //     self.command_handlers.lock().await.push(sender);
    //     receiver
    // }

    // pub async fn unregister_command_handler(&mut self, id: u32) {
    //     let mut handlers = self.command_handlers.lock().await;
    //     handlers.retain(|handler| handler.id != id);
    // }
}

async fn main_loop(
    this: Arc<DriverState>,
    mut cmd_rx: ThreadCommandReceiver,
    shutdown: Arc<Notify>,
    serial_cmd: ThreadCommandSender,
    mut serial_listener: SerialListener,
    // command_handlers: Arc<Mutex<Vec<CommandHandler>>>,
) {
    loop {
        tokio::select! {
            // We received a shutdown signal
            _ = shutdown.notified() => break,

            // We received a command from the outside
            Some((cmd, done)) = cmd_rx.recv() => main_loop_handle_command(&this, cmd, done, &serial_cmd).await,

            // The serial port emitted a frame
            Ok(frame) = serial_listener.recv() => main_loop_handle_frame(&this,frame, &serial_cmd).await
        }
    }

    println!("main task stopped")
}

async fn main_loop_handle_command(
    _this: &Arc<DriverState>,
    cmd: ThreadCommand,
    _done: oneshot::Sender<()>,
    _serial_cmd: &ThreadCommandSender,
) {
    match cmd {
        _ => {} // Ignore other commands
    }
}

async fn main_loop_handle_frame(
    this: &Arc<DriverState>,
    frame: SerialFrame,
    _serial_cmd: &ThreadCommandSender,
) {
    // TODO: Consider if we need to always handle something here
    if let SerialFrame::Command(cmd) = &frame {
        // If the awaited command registry has a matching awaiter,
        // remove it and send the command through its channel
        if let Some(channel) = this.awaited_commands.take_matching(cmd) {
            channel.send(cmd.clone()).unwrap();
        }

        // let handlers = this.command_handlers.lock().await;

        // // Invoke each handler and stop if one of them handled the command
        // for handler in handlers.iter() {
        //     if handler(cmd.clone()) {
        //         break;
        //     }
        // }
    }
}

async fn serial_loop(
    mut port: SerialPort,
    mut cmd_rx: ThreadCommandReceiver,
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
    cmd: ThreadCommand,
    done: oneshot::Sender<()>,
) {
    #[allow(irrefutable_let_patterns)]
    if let ThreadCommand::Send(frame) = cmd {
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
            println!("<< {}", hex::encode(data));
            // Try to parse the frame
            match zwave_serial::command_raw::CommandRaw::parse(data) {
                Ok((_, raw)) => {
                    // The first step of parsing was successful, ACK the frame
                    port.write(RawSerialFrame::ACK).await.unwrap();

                    // Now try to convert it into an actual command
                    match zwave_serial::command::Command::try_from(raw) {
                        Ok(cmd) => {
                            println!("received {:#?}", cmd);
                            Some(SerialFrame::Command(cmd))
                        }
                        Err(e) => {
                            println!("error: {:?}", e);
                            // TODO: Handle misformatted frames
                            None
                        }
                    }
                }
                Err(e) => {
                    println!("error: {:?}", e);
                    // Parsing failed, this means we've received garbage after all
                    port.write(RawSerialFrame::NAK).await.unwrap();
                    None
                }
            }
        }
        RawSerialFrame::Garbage(data) => {
            println!("xx: {}", hex::encode(data));
            // After receiving garbage, try to re-sync by sending NAK
            port.write(RawSerialFrame::NAK).await.unwrap();
            None
        }
        RawSerialFrame::ACK => {
            println!("<< {:?}", &frame);
            Some(SerialFrame::ACK)
        }
        RawSerialFrame::CAN => {
            println!("<< {:?}", &frame);
            Some(SerialFrame::CAN)
        }
        RawSerialFrame::NAK => {
            println!("<< {:?}", &frame);
            Some(SerialFrame::NAK)
        }
    };

    if let Some(frame) = emit {
        frame_emitter.send(frame).unwrap();
    }
}

async fn send_thread_command(
    command_sender: &ThreadCommandSender,
    cmd: ThreadCommand,
) -> Result<()> {
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
