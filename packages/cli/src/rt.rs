use std::{
    collections::VecDeque,
    io::{Read, Write},
    thread,
    time::Duration,
};

use bytes::BytesMut;
use futures::{channel::mpsc, FutureExt, SinkExt};
use serialport::TTYPort;
use zwave_core::{log, prelude::Serializable};
use zwave_driver::{Driver2, DriverEvent, DriverInput, RuntimeAdapter};
use zwave_logging::{loggers::base::BaseLogger, LogInfo, Logger};
use zwave_serial::frame::RawSerialFrame;

const BUFFER_SIZE: usize = 256;

pub struct RuntimeStatic;

pub struct Runtime {
    logger: BaseLogger,
    port: TTYPort,
    serial_in: mpsc::Sender<RawSerialFrame>,
    serial_out: mpsc::Receiver<RawSerialFrame>,
    log_receiver: mpsc::Receiver<LogInfo>,
}

impl zwave_driver::Runtime for RuntimeStatic {
    fn spawn(
        &self,
        future: futures::future::LocalBoxFuture<'static, ()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tokio::task::spawn_local(future);
        Ok(())
    }

    fn sleep(&self, duration: std::time::Duration) -> futures::future::BoxFuture<'static, ()> {
        tokio::time::sleep(duration).boxed()
    }

    // fn write_serial(&mut self, data: bytes::Bytes) {
    //     self.port
    //         .write_all(&data)
    //         .expect("failed to write to serialport");
    // }

    // fn log(&self, log: zwave_logging::LogInfo, level: zwave_core::log::Loglevel) {
    //     self.logger.log(log, level);
    // }
}

impl Runtime {
    pub fn with_adapter(logger: BaseLogger, port: TTYPort) -> (Self, RuntimeAdapter) {
        let (serial_in_tx, serial_in_rx) = mpsc::channel(16);
        let (serial_out_tx, serial_out_rx) = mpsc::channel(16);
        let (log_tx, log_rx) = mpsc::channel(16);

        (
            Self {
                logger,
                port,
                serial_in: serial_in_tx,
                serial_out: serial_out_rx,
                log_receiver: log_rx,
            },
            RuntimeAdapter {
                serial_in: serial_in_rx,
                serial_out: serial_out_tx,
                logs: log_tx,
            },
        )
    }

    pub async fn run(&mut self) {
        // let mut inputs: VecDeque<DriverInput> = VecDeque::new();
        let mut serial_in_buffer = BytesMut::zeroed(BUFFER_SIZE);
        // inputs.push_back(SerialAdapterInput::Transmit {
        //     frame: zwave_serial::frame::SerialFrame::ControlFlow(
        //         zwave_serial::frame::ControlFlow::NAK,
        //     ),
        // });
        // inputs.push_back(DriverInput::Test);

        loop {
            // Read all the available data from the serial port and handle it immediately
            serial_in_buffer.resize(BUFFER_SIZE, 0);
            match self.port.read(&mut serial_in_buffer) {
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Ok(0) => {}
                Ok(n) => {
                    serial_in_buffer.resize(n, 0);
                    while let Some(frame) =
                        RawSerialFrame::parse_mut_or_reserve(&mut serial_in_buffer)
                    {
                        self.serial_in
                            .send(frame)
                            .await
                            .expect("failed to forward frame to driver");
                    }
                }
                Err(e) => eprintln!("failed to read from serialport: {}", e),
            }

            // If the driver has something to transmit, do that before handling events
            while let Ok(Some(frame)) = self.serial_out.try_next() {
                let data = frame.as_bytes();
                self.port
                    .write_all(&data)
                    .expect("failed to write to serialport");
            }

            // If there is something to be logged, do it
            while let Ok(Some(log)) = self.log_receiver.try_next() {
                self.logger.log(log, log::Loglevel::Debug);
            }
            // while let Some(frame) = self.driver.poll_transmit() {
            //     self.port
            //         .write_all(&frame)
            //         .expect("failed to write to serialport");
            // }

            // // Check if an event needs to be handled
            // if let Some(event) = self.driver.poll_event() {
            //     match event {
            //         DriverEvent::Log { log, level } => {
            //             self.logger.log(log, level);
            //         }
            //         DriverEvent::Input { input } => {
            //             inputs.push_back(input);
            //         }
            //     }
            //     continue;
            // }

            // // Pass queued events to the driver
            // if let Some(input) = inputs.pop_front() {
            //     self.driver.handle_input(input);
            //     continue;
            // }

            // Event loop is empty, sleep for a bit
            tokio::time::sleep(Duration::from_millis(10)).await;
            // thread::sleep(Duration::from_millis(10));
        }
    }
}
