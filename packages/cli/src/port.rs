use bytes::BytesMut;
use smol::net::TcpStream;
use std::io::{self, ErrorKind, Read, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use zwave_core::prelude::Serializable;
use zwave_serial::binding::SerialBinding;
use zwave_serial::error::Result;
use zwave_serial::frame::RawSerialFrame;
use zwave_serial::serialport::FramedBinding;

type TcpFramed = FramedBinding<TcpStream>;
type SerialSender = async_channel::Sender<RawSerialFrame>;
type SerialReceiver = async_channel::Receiver<RawSerialFrame>;

const CHANNEL_CAPACITY: usize = 16;
const SERIAL_PORT_TIMEOUT: Duration = Duration::from_secs(1);

pub enum ZWavePort {
    Serial(SerialThreadPort),
    Tcp(TcpFramed),
}

impl ZWavePort {
    pub fn open_serial(path: &str) -> io::Result<Self> {
        Ok(Self::Serial(SerialThreadPort::open(path)?))
    }

    pub async fn open_tcp(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self::Tcp(FramedBinding::new(stream)))
    }
}

impl SerialBinding for ZWavePort {
    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        match self {
            ZWavePort::Serial(port) => port.write(frame).await,
            ZWavePort::Tcp(port) => port.write(frame).await,
        }
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        match self {
            ZWavePort::Serial(port) => port.read().await,
            ZWavePort::Tcp(port) => port.read().await,
        }
    }
}

// Keep blocking serial I/O at the CLI boundary by dedicating one thread to reads and one to
// writes. The rest of the stack stays executor-agnostic.
pub struct SerialThreadPort {
    outbound_tx: Option<SerialSender>,
    inbound_rx: Option<SerialReceiver>,
    shutdown: Arc<AtomicBool>,
    read_thread: Option<JoinHandle<()>>,
    write_thread: Option<JoinHandle<()>>,
}

impl SerialThreadPort {
    pub fn open(path: &str) -> io::Result<Self> {
        let write_port = open_serial_port(path)?;
        let read_port = write_port.try_clone().map_err(io::Error::from)?;

        let (outbound_tx, outbound_rx) = async_channel::bounded(CHANNEL_CAPACITY);
        let (inbound_tx, inbound_rx) = async_channel::bounded(CHANNEL_CAPACITY);
        let shutdown = Arc::new(AtomicBool::new(false));

        let read_thread = spawn_read_thread(read_port, inbound_tx, shutdown.clone())?;
        let write_thread = match spawn_write_thread(write_port, outbound_rx, shutdown.clone()) {
            Ok(thread) => thread,
            Err(err) => {
                shutdown.store(true, Ordering::Relaxed);
                outbound_tx.close();
                inbound_rx.close();
                let _ = read_thread.join();
                return Err(err);
            }
        };

        Ok(Self {
            outbound_tx: Some(outbound_tx),
            inbound_rx: Some(inbound_rx),
            shutdown,
            read_thread: Some(read_thread),
            write_thread: Some(write_thread),
        })
    }

    async fn write(&mut self, frame: RawSerialFrame) -> Result<()> {
        let Some(outbound_tx) = &self.outbound_tx else {
            return Err(channel_closed("serial write thread is unavailable"));
        };

        outbound_tx
            .send(frame)
            .await
            .map_err(|_| channel_closed("serial write thread stopped"))?;
        Ok(())
    }

    async fn read(&mut self) -> Option<RawSerialFrame> {
        let inbound_rx = self.inbound_rx.as_ref()?;
        inbound_rx.recv().await.ok()
    }
}

impl Drop for SerialThreadPort {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(outbound_tx) = self.outbound_tx.take() {
            outbound_tx.close();
        }
        if let Some(inbound_rx) = self.inbound_rx.take() {
            inbound_rx.close();
        }

        if let Some(read_thread) = self.read_thread.take() {
            let _ = read_thread.join();
        }
        if let Some(write_thread) = self.write_thread.take() {
            let _ = write_thread.join();
        }
    }
}

fn spawn_read_thread(
    mut port: Box<dyn serialport::SerialPort>,
    inbound_tx: SerialSender,
    shutdown: Arc<AtomicBool>,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("zwave-serial-read".into())
        .spawn(move || {
            let mut scratch = [0u8; 256];
            let mut buffer = BytesMut::with_capacity(256);

            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                match port.read(&mut scratch) {
                    Ok(0) => break, // EOF — port closed
                    Ok(bytes_read) => {
                        buffer.extend_from_slice(&scratch[..bytes_read]);

                        while let Some(frame) = RawSerialFrame::parse_mut_or_reserve(&mut buffer) {
                            if inbound_tx.send_blocking(frame).is_err() {
                                return;
                            }
                        }
                    }
                    Err(err)
                        if matches!(
                            err.kind(),
                            ErrorKind::Interrupted | ErrorKind::TimedOut | ErrorKind::WouldBlock
                        ) =>
                    {
                        continue;
                    }
                    Err(_) => break,
                }
            }
        })
}

fn spawn_write_thread(
    mut port: Box<dyn serialport::SerialPort>,
    outbound_rx: SerialReceiver,
    shutdown: Arc<AtomicBool>,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("zwave-serial-write".into())
        .spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                let frame = match outbound_rx.recv_blocking() {
                    Ok(frame) => frame,
                    Err(_) => break,
                };

                let mut bytes = BytesMut::new();
                frame.serialize(&mut bytes);

                if port.write_all(&bytes).is_err() {
                    break;
                }
                if port.flush().is_err() {
                    break;
                }
            }
        })
}

fn channel_closed(message: &'static str) -> zwave_serial::error::Error {
    io::Error::new(ErrorKind::BrokenPipe, message).into()
}

#[cfg(unix)]
fn open_serial_port(path: &str) -> io::Result<Box<dyn serialport::SerialPort>> {
    let mut port = serialport::new(path, 115_200)
        .timeout(SERIAL_PORT_TIMEOUT)
        .open_native()
        .map_err(io::Error::from)?;
    port.set_exclusive(false).map_err(io::Error::from)?;
    Ok(Box::new(port))
}

#[cfg(windows)]
fn open_serial_port(path: &str) -> io::Result<Box<dyn serialport::SerialPort>> {
    let port = serialport::new(path, 115_200)
        .timeout(SERIAL_PORT_TIMEOUT)
        .open_native()
        .map_err(io::Error::from)?;
    Ok(Box::new(port))
}

#[cfg(not(any(unix, windows)))]
fn open_serial_port(path: &str) -> io::Result<Box<dyn serialport::SerialPort>> {
    serialport::new(path, 115_200)
        .timeout(SERIAL_PORT_TIMEOUT)
        .open()
        .map_err(io::Error::from)
}
