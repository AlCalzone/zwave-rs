use std::thread;
use std::time::Duration;

use zwave_serial::command::GetBackgroundRssiRequest;

#[cfg(target_os = "linux")]
// const PORT: &str = "/dev/ttyUSB0";
const PORT: &str = "/dev/serial/by-id/usb-1a86_USB_Single_Serial_5479014030-if00";

#[cfg(target_os = "windows")]
const PORT: &str = "COM5";

#[tokio::main]
async fn main() {
    let mut driver = zwave_driver::Driver::new(PORT);
    println!("driver started");

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

    let result = driver
        .execute_serial_api_command(GetBackgroundRssiRequest::default())
        .await
        .unwrap();
    println!("execute result: {:?}", result);

    thread::sleep(Duration::from_millis(2000));

    drop(driver);
    println!("driver stopped");
}
