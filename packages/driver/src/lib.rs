use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use zwave_serial::binding::*;
use zwave_serial::frame::SerialFrame;
use zwave_serial::serialport::{OpenSerialPortBinding, SerialPortBinding};
use zwave_serial::error::Result;

pub struct Driver {
    // Both the port and the thread must be ManuallyDrop, because we rely on the order
    // of their destruction. To ensure this, we implement Drop below
    port: ManuallyDrop<Arc<OpenSerialPortBinding>>,
    serial_thread: ManuallyDrop<JoinHandle<()>>,
    serial_thread_signal: crossbeam::channel::Sender<()>,
}

impl Driver {
    pub fn new(path: &str) -> Self {
        let port = SerialPortBinding::new(path).open().unwrap();

        // Is stored on the Driver struct
        let port = Arc::new(port);
        // Goes into the thread
        let port2 = port.clone();
        // Used to communicate with the thread
        let (signal_tx, signal_rx) = crossbeam::channel::unbounded::<()>();

        let serial_thread = thread::spawn(move || {
            let writer = port2.writer();
            let listener = port2.listener();

            loop {
                match signal_rx.try_recv() {
                    Ok(_) | Err(crossbeam::channel::TryRecvError::Disconnected) => break,
                    Err(crossbeam::channel::TryRecvError::Empty) => {}
                }

                let frame = match listener.try_recv() {
                    Ok(frame) => Some(frame),
                    Err(crossbeam::channel::TryRecvError::Disconnected) => break,
                    Err(crossbeam::channel::TryRecvError::Empty) => None,
                };

                if let Some(frame) = frame {
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
                        writer.write(SerialFrame::ACK).unwrap();
                        if data[1] == 0x0b {
                            writer
                                .write_raw(hex::decode("01030002fe").unwrap().as_slice())
                                .unwrap();
                        }
                    }
                }

                thread::sleep(Duration::from_millis(20));
            }
        });

        Self {
            port: ManuallyDrop::new(port),
            serial_thread: ManuallyDrop::new(serial_thread),
            serial_thread_signal: signal_tx,
        }
    }

    pub fn write_raw(&self, data: &[u8]) -> Result<()> {
        self.port.writer().write_raw(data)
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        // Tell the thread to stop
        self.serial_thread_signal.send(()).unwrap();
        // Wait for it to finish
        let thread = unsafe { ManuallyDrop::take(&mut self.serial_thread) };
        thread.join().unwrap();

        // Then close the port
        let port = unsafe { ManuallyDrop::take(&mut self.port) };
        let port = Arc::try_unwrap(port).unwrap();
        port.close().unwrap();
    }
}
