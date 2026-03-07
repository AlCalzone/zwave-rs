use crate::port::ZWavePort;
use futures::StreamExt;
use tokio::{select, task};
use zwave_driver::{
    DriverActor, DriverAdapter, DriverInput, LogReceiver, SerialApiActor, SerialApiAdapter,
};
use zwave_logging::{Logger, loggers::base::BaseLogger};
use zwave_serial::binding::SerialBinding;

pub struct Runtime {
    logger: BaseLogger,
    port: ZWavePort,
    log_rx: LogReceiver,
    driver: DriverActor,
    driver_adapter: DriverAdapter,
    serial_api: SerialApiActor,
    serial_api_adapter: SerialApiAdapter,
}

impl Runtime {
    pub fn new(
        path: &str,
        logger: BaseLogger,
        log_rx: LogReceiver,
        driver: DriverActor,
        driver_adapter: DriverAdapter,
        serial_api: SerialApiActor,
        serial_api_adapter: SerialApiAdapter,
    ) -> Result<Self, anyhow::Error> {
        let open_port_result = if let Some(addr) = path.strip_prefix("tcp://") {
            ZWavePort::open_tcp(addr)
        } else {
            ZWavePort::open_serial(path)
        };

        let port = match open_port_result {
            Ok(port) => {
                // FIXME:
                // driver_logger.info(|| "serial port opened");
                port
            }
            Err(e) => {
                // FIXME:
                // driver_logger.error(|| format!("failed to open serial port: {}", e));
                return Err(e.into());
            }
        };

        Ok(Self {
            logger,
            log_rx,
            port,
            driver,
            driver_adapter,
            serial_api,
            serial_api_adapter,
        })
    }

    pub async fn run(mut self) {
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
            select! {
                biased;

                // If there is something to read from the serialport, handle it first
                Some(frame) = self.port.read() => {
                    self.serial_api_adapter
                        .serial_in
                        .try_send(frame)
                        .expect("failed to forward frame to driver");
                }

                // If the serial API has something to transmit, do that before handling events
                Some(frame) = self.serial_api_adapter.serial_out.next() => {
                    self.port.write(frame).await.expect("failed to write to serialport");
                }

                // Pass pending events from the serial API to the driver
                Some(event) = self.serial_api_adapter.event_rx.next() => {
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

                // And finally if there is something to log, do that
                Some((log, level)) = self.log_rx.next() => {
                    self.logger.log(log, level);
                }
            }
        }
    }
}
