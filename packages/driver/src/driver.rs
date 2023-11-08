use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, Notify};
use tokio::task::JoinHandle;
use zwave_serial::binding::*;
use zwave_serial::error::Result;
use zwave_serial::frame::SerialFrame;
use zwave_serial::serialport::SerialPort;

enum ThreadCommand {
    Send(SerialFrame),
}

type ThreadCommandSender = mpsc::Sender<(ThreadCommand, oneshot::Sender<()>)>;
type SerialListener = broadcast::Receiver<SerialFrame>;

#[allow(dead_code)]
pub struct Driver {
    serial_task: JoinHandle<()>,
    main_task: JoinHandle<()>,
    main_cmd: ThreadCommandSender,
    main_task_shutdown: Arc<Notify>,
    serial_cmd: ThreadCommandSender,
    serial_listener: SerialListener,
    serial_task_shutdown: Arc<Notify>,
}

impl Driver {
    pub fn new(path: &str) -> Self {
        // The serial task owns the serial port. All communication needs to go through that task.
        let mut port = SerialPort::new(path).unwrap();
        // To control it, we send a thread command along with a "callback" oneshot channel to the task.
        let (serial_cmd_tx, mut serial_cmd_rx) =
            mpsc::channel::<(ThreadCommand, oneshot::Sender<()>)>(100);
        // The listener is used to receive frames from the serial port
        let (serial_listener_tx, serial_listener_rx) = broadcast::channel::<SerialFrame>(100);
        let serial_task_shutdown = Arc::new(Notify::new());
        let serial_task_shutdown2 = serial_task_shutdown.clone();

        // The main logic happens in another task that owns the internal state.
        // To control it, we need another channel.
        let (main_cmd_tx, mut main_cmd_rx) =
            mpsc::channel::<(ThreadCommand, oneshot::Sender<()>)>(100);
        let main_serial_cmd = serial_cmd_tx.clone();
        let mut main_serial_listener = serial_listener_tx.subscribe();
        let main_task_shutdown = Arc::new(Notify::new());
        let main_task_shutdown2 = main_task_shutdown.clone();

        let main_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // We received a command from the outside
                    _ = main_task_shutdown2.notified() => {
                        // Exit the task
                        break;
                    }
                    Some((cmd, tx)) = main_cmd_rx.recv() => {
                        match cmd {
                            _ => {}, // Ignore other commands
                        }
                    }

                    // The serial port emitted a frame
                    Ok(frame) = main_serial_listener.recv() => {
                        match &frame {
                            SerialFrame::Data(data) => {
                                println!("<< {}", hex::encode(&data));
                            }
                            SerialFrame::Garbage(data) => {
                                println!("DISCARDED: {}", hex::encode(&data));
                            }
                            SerialFrame::ACK | SerialFrame::CAN | SerialFrame::NAK => {
                                println!("<< {:?}", &frame);
                            }
                        }

                    if let SerialFrame::Data(data) = &frame {
                        match zwave_serial::command::Command::parse(data) {
                            Ok((_, command)) => println!("received {:#?}", command),
                            Err(e) => println!("error: {:?}", e),
                        }
                        // Send ACK
                        send_thread_command(&main_serial_cmd, ThreadCommand::Send(SerialFrame::ACK)).await.unwrap();
                        if data[1] == 0x0b {
                            send_thread_command(&main_serial_cmd, ThreadCommand::Send(SerialFrame::Data(hex::decode("01030002fe").unwrap()))).await.unwrap();

                        }
                    }

                    }
                }
            }

            println!("main task stopped")
        });

        // Run the serial communication as a background task
        let serial_task = tokio::spawn(async move {
            // Whatever happens first gets handled first.
            loop {
                tokio::select! {
                    // We received a command from the outside
                    _ = serial_task_shutdown2.notified() => {
                        // Exit the task
                        break;
                    }
                    Some((cmd, tx)) = serial_cmd_rx.recv() => {
                        match cmd {
                            ThreadCommand::Send(frame) => {
                                port.write(frame).await.unwrap();
                                tx.send(()).unwrap();
                            }

                            // Ignore other commands
                            #[allow(unreachable_patterns)]
                            _ => {},
                        }
                    }
                    // We received a frame from the serial port
                    Some(frame) = port.read() => {
                        serial_listener_tx.send(frame).unwrap();
                    }
                }
            }

            println!("serial task stopped")
        });

        Self {
            main_task,
            main_cmd: main_cmd_tx,
            main_task_shutdown,
            serial_task,
            serial_cmd: serial_cmd_tx,
            serial_task_shutdown,
            serial_listener: serial_listener_rx,
        }
    }

    pub async fn write(&mut self, frame: SerialFrame) -> Result<()> {
        send_thread_command(&self.serial_cmd, ThreadCommand::Send(frame)).await
    }
}

async fn send_thread_command(
    command_sender: &ThreadCommandSender,
    cmd: ThreadCommand,
) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    command_sender.send((cmd, tx)).await.unwrap();
    rx.await.unwrap();
    Ok(())
}

// fn send_blocking_thread_command(
//     command_sender: &ThreadCommandSender,
//     cmd: ThreadCommand,
// ) -> Result<()> {
//     let (tx, rx) = oneshot::channel();
//     command_sender.blocking_send((cmd, tx)).unwrap();
//     rx.blocking_recv().unwrap();
//     Ok(())
// }

impl Drop for Driver {
    fn drop(&mut self) {
        // We need to stop the background tasks, otherwise they will stick around until the process exits
        self.serial_task_shutdown.notify_one();
        self.main_task_shutdown.notify_one();
    }
}
