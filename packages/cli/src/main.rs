use std::thread;
use std::time::Duration;
use zwave_serial::command::GetProtocolVersionRequest;

#[cfg(target_os = "linux")]
const PORT: &str = "/dev/ttyUSB0";

#[cfg(target_os = "windows")]
const PORT: &str = "COM5";

#[tokio::main]
async fn main() {
    let mut driver = zwave_driver::Driver::new(PORT);
    println!("driver started");

    #[allow(clippy::unnecessary_fallible_conversions)]
    driver
        .write_serial(GetProtocolVersionRequest::new().try_into().unwrap())
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(2000));

    drop(driver);
    println!("driver stopped");
}
