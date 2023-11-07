use std::thread;
use std::time::Duration;

fn main() {
    let driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started");

    driver
        .write_raw(&hex::decode("01030008f4").unwrap())
        .unwrap();

    thread::sleep(Duration::from_millis(10000));

    drop(driver);
    println!("driver stopped");

    let driver = zwave_driver::Driver::new("/dev/ttyUSB0");
    println!("driver started again");

    thread::sleep(Duration::from_millis(10000));

    drop(driver);
    println!("driver stopped again");
}
