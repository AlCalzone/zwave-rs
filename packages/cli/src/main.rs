use std::time::Duration;
use tokio::task;
use zwave_cc::{
    commandclass::{BasicCCGet, SecurityCCCommandEncapsulation},
    prelude::*,
};
use zwave_core::log::Loglevel;
use zwave_driver::{Controller, SecurityKeys};
use zwave_logging::loggers::base::BaseLogger;

mod rt;
use rt::Runtime;

#[cfg(target_os = "linux")]
const PORT: &str = "/dev/ttyUSB0";
// const PORT: &str = "tcp://Z-Net-R2v2.local:2001";

#[cfg(target_os = "windows")]
const PORT: &str = "COM6";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    // let options = DriverOptions::builder()
    //     .path(PORT)
    //     // .loglevel(Loglevel::Silly)
    //     .security_keys(SecurityKeys {
    //         s0_legacy: Some(vec![
    //             0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    //             0x0E, 0x0F, 0x10,
    //         ]),
    //         ..Default::default()
    //     })
    //     .build();

    let security_keys = SecurityKeys::builder()
        .s0_legacy(Some(vec![
            0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]))
        .build();

    let logger = BaseLogger {
        level: Loglevel::Debug,
        writer: Box::new(termcolor::StandardStream::stdout(
            termcolor::ColorChoice::Auto,
        )),
        formatter: Box::new(zwave_logging::formatters::DefaultFormatter::new()),
    };

    // FIXME:
    //     driver_logger.info(|| format!("opening serial port {}", PORT));

    let (log_tx, log_rx) = futures::channel::mpsc::channel(16);

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
    )?;

    let local = task::LocalSet::new();
    local
        .run_until(async move {
            let main = task::spawn_local(async move {
                runtime.run().await;
            });

            let controller = Controller::new(&driver);
            let controller: Controller<'_, zwave_driver::Ready> =
                controller.interview().await.unwrap();

            tokio::time::sleep(Duration::from_secs(3)).await;

            let node = controller.get_node(&6u8.into()).unwrap();

            let cc: CC = BasicCCGet::default().into();
            let cc: CC = SecurityCCCommandEncapsulation::new(cc).into();
            let cc = cc.with_destination(node.id().into());
            let result = driver.exec_node_command(&cc, None).await;
            println!("result: {:?}", result);

            // let ping_result = controller
            //     .get_node(&4u8.into())
            //     .unwrap()
            //     .ping()
            //     .await
            //     .unwrap();
            // tokio::time::sleep(Duration::from_millis(100)).await;

            // println!("ping result: {:?}", ping_result);

            // println!("home ID: {:?}", controller.home_id());

            // let result = driver
            //     .exec_node_command(&NoOperationCC {}.with_destination(3u8.into()).into(), None)
            //     .await;
            // println!("result: {:?}", result);

            // // let cmd = zwave_serial::command::GetControllerVersionRequest::default();
            // // let result = api.execute_serial_api_command(cmd).await.unwrap();
            // let result = api.get_controller_version(None).await.unwrap();
            // println!("result: {:?}", result);

            tokio::time::sleep(Duration::from_secs(3)).await;
            println!("Bye");
            main.abort();
            // driver_future.abort();
        })
        .await;

    // let driver = zwave_driver::Driver::new(options).expect("Failed to create driver");

    // let driver = driver.init().await.expect("Failed to initialize driver");

    // tokio::time::sleep(Duration::from_millis(1000)).await;

    // let cc = SecurityCCCommandEncapsulation::new(
    //     BasicCCSet::builder()
    //         .target_value(zwave_core::values::LevelSet::Level(55))
    //         .build()
    //         .into(),
    // )
    // .with_destination(2u8.into());
    // driver.exec_node_command(&cc.into(), None).await.unwrap();

    // tokio::time::sleep(Duration::from_millis(60000)).await;

    // driver.interview_nodes().await.expect("Failed to interview nodes");
    // driver.log().info(|| "all nodes interviewed");

    // // node2.ping().await.unwrap();

    // let node = driver.get_node(&NodeId::new(2u8)).expect("Node not found");
    // let nonce = node.cc_api().security().get_nonce().await.unwrap();
    // println!("nonce: {:#?}", &nonce);

    // node.cc_api().basic().set(LevelSet::Off).await.unwrap();

    // let ping_result = node.ping().await.unwrap();
    // println!("ping result: {:?}", ping_result);

    // driver
    //     .register_command_handler(Box::new(|cmd| {
    //         if cmd.function_type() == FunctionType::AddNodeToNetwork {
    //             println!("received add node to network request: {:?}", cmd);
    //             return true;
    //         }
    //         false
    //     }))
    //     .await;

    // driver
    //     .register_command_handler(Box::new(|cmd| {
    //         if cmd.function_type() == FunctionType::GetProtocolVersion {
    //             println!("received protocol version: {:?}", cmd);
    //             return true;
    //         }
    //         false
    //     }))
    //     .await;

    // #[allow(clippy::unnecessary_fallible_conversions)]
    // driver
    //     .write_serial(GetProtocolVersionRequest::new().try_into().unwrap())
    //     .await
    //     .unwrap();

    // println!("sent protocol version request, waiting for response");

    // match driver
    //     .await_command(
    //         Box::new(|cmd| cmd.function_type() == FunctionType::GetProtocolVersion),
    //         Some(Duration::from_secs(2)),
    //     )
    //     .await
    // {
    //     Some(cmd) => println!("AWAITING received protocol version: {:?}", cmd),
    //     None => println!("timed out waiting for protocol version"),
    // }

    // let mut failures: Vec<SerialApiMachineResult> = Vec::new();
    // let mut ok = 0;
    // let mut on = false;
    // for i in 1..=1 {
    //     println!("sending data");
    //     let cmd = SendDataRequest::builder()
    //         .node_id(7)
    //         .transmit_options(TransmitOptions::new().ack(true).no_route(true))
    //         .payload(vec![
    //             // 0x00, // PING
    //             0x20, // Basic CC
    //             0x02, // Basic Get
    //                   // if on { 0xFF } else { 0x00 },
    //         ])
    //         .build()
    //         .unwrap();
    //     let result = driver.execute_serial_api_command(cmd).await.unwrap();

    //     println!("execute result: {:?}", result);
    //     match result {
    //         SerialApiMachineResult::Success(_) => {
    //             ok += 1;
    //             println!("Test {i} passed");
    //         }
    //         _ => {
    //             failures.push(result);
    //             println!("Test {i} failed");
    //             break;
    //         }
    //     }
    //     on = !on;
    //     tokio::time::sleep(Duration::from_millis(250)).await;
    // }

    // println!("{} tests PASSED, {} tests FAILED", ok, failures.len());
    // if !failures.is_empty() {
    //     println!("Failures: {:?}", failures);
    // }

    // let cc = BinarySwitchCCSet::builder()
    //     .target_value(BinarySet::Off)
    //     .build();
    // let result = driver
    //     .exec_node_command(&cc.with_destination(2.into()), None)
    //     .await;

    // // let cmd = SendDataRequest::builder()
    // //     .node_id(2)
    // //     .command(
    // //         BasicCCSet {
    // //             target_value: LevelSet::Off,
    // //         }
    // //         .into(),
    // //     )
    // //     .build();

    // // let result = driver.execute_serial_api_command(cmd).await.unwrap();
    // println!("execute result: {:?}", result);

    // tokio::time::sleep(Duration::from_millis(1000)).await;

    // drop(driver);
    // println!("driver stopped");
    Ok(())
}
