use crate::port::ZWavePort;
use futures::{FutureExt, StreamExt, select_biased};
use smol::LocalExecutor;
use zwave_driver::{
    DriverActor, DriverAdapter, DriverInput, LogReceiver, SerialApiActor, SerialApiAdapter,
};
use zwave_logging::{Logger, loggers::base::BaseLogger};
use zwave_serial::binding::SerialBinding;
use zwave_serial::frame::RawSerialFrame;

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
    pub async fn new(
        path: &str,
        logger: BaseLogger,
        log_rx: LogReceiver,
        driver: DriverActor,
        driver_adapter: DriverAdapter,
        serial_api: SerialApiActor,
        serial_api_adapter: SerialApiAdapter,
    ) -> Result<Self, anyhow::Error> {
        let open_port_result = if let Some(addr) = path.strip_prefix("tcp://") {
            ZWavePort::open_tcp(addr).await
        } else {
            ZWavePort::open_serial(path)
        };

        let port = match open_port_result {
            Ok(port) => port,
            Err(e) => return Err(e.into()),
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

    pub fn spawn(self, local: &LocalExecutor<'_>) -> smol::Task<()> {
        let Self {
            logger,
            port,
            log_rx,
            driver,
            driver_adapter,
            serial_api,
            serial_api_adapter,
        } = self;

        // Start the driver and serial API actors.
        let driver_task = local.spawn(async move {
            let mut driver = driver;
            driver.run().await;
        });
        let serial_api_task = local.spawn(async move {
            let mut serial_api = serial_api;
            serial_api.run().await;
        });

        local.spawn(async move {
            let mut logger = logger;
            let mut port = port;
            let mut log_rx = log_rx;
            let mut driver_adapter = driver_adapter;
            let mut serial_api_adapter = serial_api_adapter;

            loop {
                select_biased! {
                    // If there is something to read from the serialport, handle it first.
                    serial_in = port.read().fuse() => {
                        let Some(frame) = serial_in else {
                            // Port closed => quit
                            break;
                        };
                        if !forward_serial_frame(&mut serial_api_adapter, frame) {
                            // Channel probably closed => quit
                            break;
                        }
                    }

                    // If the serial API has something to transmit, do that before handling events.
                    serial_out = serial_api_adapter.serial_out.next().fuse() => {
                        let Some(frame) = serial_out else {
                            // Channel closed => quit
                            break;
                        };
                        if port.write(frame).await.is_err() {
                            // Port closed => quit
                            break;
                        }
                    }

                    // Pass pending events from the serial API to the driver.
                    event_rx = serial_api_adapter.event_rx.next().fuse() => {
                        let Some(event) = event_rx else {
                            // Channel closed => quit
                            break;
                        };
                        match event {
                            zwave_driver::SerialApiEvent::Unsolicited { command } => {
                                // Forward unsolicited commands to the driver
                                if !forward_unsolicited(&mut driver_adapter, command) {
                                    // Channel probably closed => quit
                                    break;
                                }
                            }
                        }
                    }

                    // And finally if there is something to log, do that.
                    log = log_rx.next().fuse() => {
                        let Some((log, level)) = log else {
                            // Channel closed => quit
                            break;
                        };
                        logger.log(log, level);
                    }
                }
            }

            let _ = driver_task.cancel().await;
            let _ = serial_api_task.cancel().await;
        })
    }
}

fn forward_serial_frame(serial_api_adapter: &mut SerialApiAdapter, frame: RawSerialFrame) -> bool {
    match serial_api_adapter.serial_in.try_send(frame) {
        Ok(()) => true,
        Err(err) if err.is_disconnected() => false,
        Err(_) => panic!("failed to forward frame to driver"),
    }
}

fn forward_unsolicited(
    driver_adapter: &mut DriverAdapter,
    command: zwave_serial::command::Command,
) -> bool {
    match driver_adapter
        .input_tx
        .try_send(DriverInput::Unsolicited { command })
    {
        Ok(()) => true,
        Err(err) if err.is_disconnected() => false,
        Err(_) => panic!("failed to forward unsolicited command to driver"),
    }
}
