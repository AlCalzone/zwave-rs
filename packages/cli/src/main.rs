use std::thread;
use std::time::Duration;
use zwave_driver::SerialApiMachineResult;
use zwave_serial::command::{
    Command, CommandBase, GetProtocolVersionRequest, SendDataCallback, SendDataRequest,
};

#[cfg(target_os = "linux")]
const PORT: &str = "/dev/ttyUSB0";

// const PORT: &str = "/dev/serial/by-id/usb-0658_0200_E4051302-4A02-010A-1407-031500A31880-if00";

#[cfg(target_os = "windows")]
const PORT: &str = "COM5";

#[tokio::main]
async fn main() {
    let driver = zwave_driver::Driver::new(PORT);
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

    let mut failures: Vec<SerialApiMachineResult> = Vec::new();
    let mut ok = 0;
    for i in 1..=250 {
        println!("sending data");
        let result = driver
            .execute_serial_api_command(SendDataRequest::new())
            .await
            .unwrap();

        println!("execute result: {:?}", result);
        match result {
            SerialApiMachineResult::CallbackNOK(Command::SendDataCallback(cb)) if !cb.is_ok() => {
                ok += 1;
                println!("Test {i} passed");
            }
            _ => {
                failures.push(result);
                println!("Test {i} failed");
                break;
            }
        }
    }

    println!("{} tests PASSED, {} tests FAILED", ok, failures.len());
    if !failures.is_empty() {
        println!("Failures: {:?}", failures);
    }

    let result = driver
        .execute_serial_api_command(SendDataRequest::new())
        .await
        .unwrap();
    println!("execute result: {:?}", result);

    thread::sleep(Duration::from_millis(2000));

    drop(driver);
    println!("driver stopped");
}
