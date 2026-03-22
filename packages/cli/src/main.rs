use std::time::Duration;
use zwave_cc::{commandclass, prelude::CCAddressable};
use zwave_core::log::Loglevel;
use zwave_driver::{Controller, SecurityKeys};
use zwave_logging::loggers::base::BaseLogger;

mod port;
mod rt;
use rt::Runtime;

#[cfg(target_os = "linux")]
const PORT: &str = "/dev/serial/by-id/usb-Nabu_Casa_ZWA-2_8CBFEA8F6974-if00";
// const PORT: &str = "tcp://Z-Net-R2v2.local:2001";

#[cfg(target_os = "windows")]
const PORT: &str = "COM6";

fn main() -> Result<(), anyhow::Error> {
    let local = smol::LocalExecutor::new();

    smol::block_on(local.run(async {
        let security_keys = SecurityKeys::builder()
            .s0_legacy([
                0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F, 0x10,
            ])
            .build();

        let logger = BaseLogger {
            level: Loglevel::Debug,
            writer: Box::new(termcolor::StandardStream::stdout(
                termcolor::ColorChoice::Auto,
            )),
            formatter: Box::new(zwave_logging::formatters::DefaultFormatter::new()),
        };

        let (log_tx, log_rx) = zwave_pal::channel::channel(16);

        let (serial_api, serial_api_actor, serial_api_adapter) =
            zwave_driver::SerialApi::new(log_tx.clone());
        let (driver, driver_actor, driver_adapter) =
            zwave_driver::Driver::new(&serial_api, log_tx, security_keys);

        let runtime = Runtime::new(
            PORT,
            logger,
            log_rx,
            driver_actor,
            driver_adapter,
            serial_api_actor,
            serial_api_adapter,
        )
        .await?;

        let runtime_task = runtime.spawn(&local);

        let controller = Controller::new(&driver);
        let _controller: Controller<'_, zwave_driver::Ready> =
            controller.interview().await.unwrap();

        smol::Timer::after(Duration::from_secs(1)).await;

        let cc = commandclass::BasicCCSet::builder()
            .target_value(zwave_core::values::LevelSet::On)
            .build()
            .with_destination(11u8.into());
        let result = driver.exec_node_command(&cc.into(), None).await;
        println!("result: {:?}", result);

        smol::Timer::after(Duration::from_secs(1)).await;
        println!("Bye");

        let _ = runtime_task.cancel().await;

        Ok(())
    }))
}
