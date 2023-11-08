use std::thread;
use std::time::Duration;
use zwave_serial::frame::SerialFrame;

#[tokio::main]
async fn main() {
    let mut driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started");

    driver
        .write(SerialFrame::Data(hex::decode("01030008f4").unwrap()))
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(10000));

    drop(driver);
    println!("driver stopped");

    let mut driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started again");

    driver
        .write(SerialFrame::Data(hex::decode("01030008f4").unwrap()))
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(10000));

    drop(driver);
    println!("driver stopped again");
}
