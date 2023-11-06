use crate::binding::{OpenBinding, Binding, SerialWriter};
use crate::error::Result;
use crate::frame::SerialFrame;
use bytes::{Buf, BytesMut};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
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
    Write(Vec<u8>),
}

#[derive(Debug)]
pub struct OpenSerialPortBinding {
    builder: SerialPortBuilder,
    thread: JoinHandle<()>,
    command_tx: Sender<ThreadCommand>,
    frames_rx: Receiver<SerialFrame>,
}

impl Binding for SerialPortBinding {
    type Open = OpenSerialPortBinding;

    fn new(path: &str) -> Self {
        let builder = serialport::new(path, 115_200).timeout(Duration::from_millis(10));
        return Self { builder };
    }

    fn open(self) -> Result<Self::Open> {
        let mut port = self.builder.clone().open()?;
        // Create a channel to communicate with the thread
        let (command_tx, command_rx) = crossbeam_channel::unbounded::<ThreadCommand>();
        // Create a channel to allow callers to listen for frames
        let (frames_tx, frames_rx) = crossbeam_channel::unbounded::<SerialFrame>();

        let thread = thread::spawn(move || {
            // parse_buf keeps track of the data that has been read from the serial port
            // and allows easy appending of new data
            let mut parse_buf = BytesMut::with_capacity(512);
            // serial_buf is needed to read data from the serial port
            let mut serial_buf: Vec<u8> = vec![0; 256];

            loop {
                let cmd = match command_rx.try_recv() {
                    Ok(ThreadCommand::Stop) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => None,
                    Ok(cmd) => Some(cmd),
                };

                // Try to read from serial port and store the data into the serial buffer at the offset
                match port.read(&mut serial_buf) {
                    Ok(t) => {
                        parse_buf.extend_from_slice(&serial_buf[..t]);
                        while let Ok((remaining, frame)) =
                            SerialFrame::parse(&parse_buf.to_vec())
                        {
                            // Emit the data to the listener and exit when there isn't one anymore
                            if frames_tx.send(frame).is_err() {
                                break;
                            }

                            let bytes_read = parse_buf.len() - remaining.len();
                            parse_buf.advance(bytes_read);
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
                if let Some(ThreadCommand::Write(data)) = cmd {
                    port.write_all(&data).unwrap();
                }
            }
        });
        return Ok(OpenSerialPortBinding {
            builder: self.builder,
            thread,
            command_tx,
            frames_rx,
        });
    }
}

struct SerialPortWriter {
    sender: Sender<ThreadCommand>,
}

impl SerialWriter<'_> for SerialPortWriter {
    fn write_raw(&self, data: impl AsRef<[u8]>) -> Result<()> {
        let data = data.as_ref();
        if data.len() > 1 {
            println!(">> {}", hex::encode(&data));
        }

        self.sender
            .send(ThreadCommand::Write(data.as_ref().to_vec()))
            .unwrap();
        Ok(())
    }

    fn write(&self, frame: SerialFrame) -> Result<()> {
        let data = frame.as_ref();
        match &frame {
            SerialFrame::Data(_) => {
                println!(">> {}", hex::encode(&data));
            }
            SerialFrame::ACK | SerialFrame::CAN | SerialFrame::NAK => {
                println!(">> {:?}", &frame);
            }
            _ => (),
        }

        self.write_raw(data)
    }
}

impl Clone for SerialPortWriter {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl OpenBinding for OpenSerialPortBinding {
    type Closed = SerialPortBinding;

    fn close(self) -> Result<Self::Closed> {
        // Stop the thread and wait for it. We have to expect that the
        // thread has already exited due to no listeners being active anymore,
        // so ignore a potential Error
        let _ = self.command_tx.send(ThreadCommand::Stop);
        self.thread.join().unwrap();

        Ok(SerialPortBinding {
            builder: self.builder,
        })
    }

    fn writer<'a>(&self) -> impl crate::binding::SerialWriter<'_> + Clone {
        SerialPortWriter {
            sender: self.command_tx.clone(),
        }
    }

    fn listener(&self) -> crate::binding::SerialListener {
        self.frames_rx.clone()
    }
}
