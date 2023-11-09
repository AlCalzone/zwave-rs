use std::thread;
use std::time::Duration;
use zwave_serial::command::{GetSerialApiInitDataRequest, SoftResetRequest};
use zwave_serial::frame::SerialFrame;

#[tokio::main]
async fn main() {
    let mut driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started");

    driver
        .write_serial(GetSerialApiInitDataRequest::new().try_into().unwrap())
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(5000));

    drop(driver);
    println!("driver stopped");

    let mut driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started again");

    driver
        .write_serial(SoftResetRequest::new().try_into().unwrap())
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(5000));

    drop(driver);
    println!("driver stopped again");
}
