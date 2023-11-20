use std::thread;
use std::time::Duration;
use zwave_core::definitions::FunctionType;
use zwave_serial::command::{CommandBase, GetProtocolVersionRequest};

#[cfg(target_os = "linux")]
const PORT: &str = "/dev/serial/by-id/usb-0658_0200_E4051302-4A02-010A-1407-031500A31880-if00";

#[cfg(target_os = "windows")]
const PORT: &str = "COM5";

#[tokio::main]
async fn main() {
    let mut driver = zwave_driver::Driver::new(PORT);
    println!("driver started");

    driver
        .register_command_handler(Box::new(|cmd| {
            if cmd.function_type() == FunctionType::AddNodeToNetwork {
                println!("received add node to network request: {:?}", cmd);
                return true;
            }
            false
        }))
        .await;

    driver
        .register_command_handler(Box::new(|cmd| {
            if cmd.function_type() == FunctionType::GetProtocolVersion {
                println!("received protocol version: {:?}", cmd);
                return true;
            }
            false
        }))
        .await;

    #[allow(clippy::unnecessary_fallible_conversions)]
    driver
        .write_serial(GetProtocolVersionRequest::new().try_into().unwrap())
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(2000));

    drop(driver);
    println!("driver stopped");
}
