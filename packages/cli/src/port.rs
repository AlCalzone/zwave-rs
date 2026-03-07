use tokio::net::TcpStream;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use zwave_serial::binding::SerialBinding;
use zwave_serial::error::Result;
use zwave_serial::frame::RawSerialFrame;
use zwave_serial::serialport::FramedBinding;

type SerialFramed = FramedBinding<tokio_util::compat::Compat<tokio_serial::SerialStream>>;
type TcpFramed = FramedBinding<tokio_util::compat::Compat<TcpStream>>;

pub enum ZWavePort {
    Serial(SerialFramed),
    Tcp(TcpFramed),
}

impl ZWavePort {
    pub fn open_serial(path: &str) -> std::io::Result<Self> {
        #[allow(unused_mut)]
        let mut port = tokio_serial::new(path, 115_200).open_native_async()?;
        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");
        Ok(ZWavePort::Serial(FramedBinding::new(port.compat())))
    }

    pub fn open_tcp(addr: &str) -> std::io::Result<Self> {
        let stream = std::net::TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        let stream = TcpStream::from_std(stream)?;
        Ok(ZWavePort::Tcp(FramedBinding::new(stream.compat())))
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
