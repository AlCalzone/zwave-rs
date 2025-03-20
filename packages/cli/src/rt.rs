use bytes::BytesMut;
use serialport::TTYPort;
use std::{
    io::{Read, Write},
    time::Duration,
};
use tokio::task;
use zwave_core::prelude::Serializable;
use zwave_driver::{
    DriverActor, DriverAdapter, DriverInput, LogReceiver, SerialApiActor, SerialApiAdapter,
};
use zwave_logging::{loggers::base::BaseLogger, Logger};
use zwave_serial::frame::RawSerialFrame;

const BUFFER_SIZE: usize = 256;

pub struct Runtime {
    logger: BaseLogger,
    port: TTYPort,
    log_rx: LogReceiver,
    driver: DriverActor,
    driver_adapter: DriverAdapter,
    serial_api: SerialApiActor,
    serial_api_adapter: SerialApiAdapter,
}

impl Runtime {
    pub fn new(
        port: TTYPort,
        logger: BaseLogger,
        log_rx: LogReceiver,
        driver: DriverActor,
        driver_adapter: DriverAdapter,
        serial_api: SerialApiActor,
        serial_api_adapter: SerialApiAdapter,
    ) -> Self {
        Self {
            logger,
            log_rx,
            port,
            driver,
            driver_adapter,
            serial_api,
            serial_api_adapter,
        }
    }

    pub async fn run(mut self) {
        let mut serial_in_buffer = BytesMut::zeroed(BUFFER_SIZE);

        let mut driver = self.driver;
        let mut serial_api = self.serial_api;

        // Start the driver and serial API actors
        task::spawn_local(async move {
            driver.run().await;
        });
        task::spawn_local(async move {
            serial_api.run().await;
        });

        loop {
            // FIXME: Migrate to async serial port and select! macro

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
                        self.serial_api_adapter
                            .serial_in
                            .try_send(frame)
                            .expect("failed to forward frame to driver");
                    }
                }
                Err(e) => eprintln!("failed to read from serialport: {}", e),
            }

            // If the serial API has something to transmit, do that before handling events
            while let Ok(Some(frame)) = self.serial_api_adapter.serial_out.try_next() {
                let data = frame.as_bytes();
                self.port
                    .write_all(&data)
                    .expect("failed to write to serialport");
            }

            // Pass pending events from the serial API to the driver
            while let Ok(Some(event)) = self.serial_api_adapter.event_rx.try_next() {
                match event {
                    zwave_driver::SerialApiEvent::Unsolicited { command } => {
                        // Forward unsolited commands to the driver
                        self.driver_adapter
                            .input_tx
                            .try_send(DriverInput::Unsolicited { command })
                            .expect("failed to forward unsolicited command to driver");
                    }
                }
            }

            // If there is something to be logged, do it
            while let Ok(Some((log, level))) = self.log_rx.try_next() {
                self.logger.log(log, level);
            }

            // Event loop is empty, sleep for a bit
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
