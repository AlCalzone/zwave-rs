use std::thread;
use std::time::Duration;
use zwave_serial::command::{
    GetControllerVersionRequest, GetProtocolVersionRequest, GetSerialApiInitDataRequest,
};

#[tokio::main]
async fn main() {
    let mut driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started");

    driver
        .write_serial(GetProtocolVersionRequest::new().try_into().unwrap())
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(2000));

    drop(driver);
    println!("driver stopped");
}
