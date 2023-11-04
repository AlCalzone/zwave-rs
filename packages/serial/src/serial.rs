use crate::common::{OpenPortBinding, PortBinding, SerialAPIFrame, SerialAPIListener};
use crate::error::Result;
use crossbeam_channel::{Sender, TryRecvError};
use serialport::SerialPortBuilder;
use std::io::{self};
use std::thread;
use std::{thread::JoinHandle, time::Duration};

pub struct SerialPortBinding {
    builder: SerialPortBuilder,
}

#[derive(Debug)]
enum ThreadCommand {
    Stop,
    Send(Vec<u8>),
}

#[derive(Debug)]
pub struct OpenSerialPortBinding {
    builder: SerialPortBuilder,
    thread: JoinHandle<()>,
    thread_signal: Sender<ThreadCommand>,
}

impl PortBinding for SerialPortBinding {
    type Open = OpenSerialPortBinding;

    fn new(path: &str) -> Self {
        let builder = serialport::new(path, 115_200).timeout(Duration::from_millis(10));
        return Self { builder };
    }

    fn open(self, listener: SerialAPIListener) -> Result<Self::Open> {
        let mut port = self.builder.clone().open()?;
        let (tx, rx) = crossbeam_channel::unbounded::<ThreadCommand>();

        let thread = thread::spawn(move || {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            // Offset marks the end of the existing data
            let mut offset: usize = 0;
            loop {
                let cmd = match rx.try_recv() {
                    Ok(ThreadCommand::Stop) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => None,
                    Ok(cmd) => Some(cmd),
                };

                if !cmd.is_none() {
                    println!("Got command {:?}", cmd);
                }

                // Try to read from serial port and store the data into the serial buffer at the offset
                match port.read(&mut serial_buf[offset..]) {
                    Ok(t) => {
                        // End of data is now t bytes further
                        offset += t;

                        while let Some((frame, skip)) =
                            SerialAPIFrame::try_parse(&serial_buf[..offset])
                        {
                            match &frame {
                                SerialAPIFrame::Control(data) => {
                                    println!("<< {:?}", data);
                                }
                                SerialAPIFrame::Command(data) => {
                                    println!("<< {}", hex::encode(data));
                                }
                                SerialAPIFrame::Garbage(data) => {
                                    println!("DISCARDED: {}", hex::encode(data));
                                }
                            }

                            // Emit the data to the listener and exit when there isn't one anymore
                            if listener.send(frame).is_err() {
                                break;
                            }

                            // Move the remaining data in the serial buffer to the start
                            // SSSSSSxx0000
                            serial_buf.copy_within(skip..offset, 0);
                            // xxSSSSxx0000
                            // and fill the rest with 0
                            serial_buf[(offset - skip)..offset].fill(0);
                            // xx0000000000
                            offset -= skip;
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        // No data to read, continue
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break;
                    }
                }

                // When we're done or there's nothing to read, handle pending writes
                if let Some(ThreadCommand::Send(data)) = cmd {
                    port.write_all(&data).unwrap();
                    println!(">> {}", hex::encode(data));
                }
            }
        });
        return Ok(OpenSerialPortBinding {
            builder: self.builder,
            thread,
            thread_signal: tx,
        });
    }
}

impl OpenPortBinding for OpenSerialPortBinding {
    type Closed = SerialPortBinding;

    fn close(self) -> Result<Self::Closed> {
        // Stop the thread and wait for it. We have to expect that the
        // thread has already exited due to no listeners being active anymore,
        // so ignore a potential Error
        let _ = self.thread_signal.send(ThreadCommand::Stop);
        self.thread.join().unwrap();

        Ok(SerialPortBinding {
            builder: self.builder,
        })
    }

    fn write(&mut self, data: Vec<u8>) -> Result<()> {
        self.thread_signal.send(ThreadCommand::Send(data)).unwrap();

        // TODO: Handle errors
        Ok(())
    }
}
