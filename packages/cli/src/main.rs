use serial::common::*;
use serial::serial::SerialPortBinding;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let port = SerialPortBinding::new("/dev/ttyUSB0");

    let (sender, receiver) = crossbeam_channel::unbounded::<SerialAPIFrame>();

    let mut port = port.open(sender).unwrap();

    port.write(hex::decode("01030008f4").unwrap()).unwrap();

    let port_ref = Arc::new(Mutex::new(port));
    let shared_port: Arc<Mutex<serial::serial::OpenSerialPortBinding>> = port_ref.clone();
    let shared_receiver = receiver.clone();
    {
        thread::spawn(move || {
            for msg in shared_receiver.iter().take(2) {
                {
                    let mut port_ref = shared_port.lock().unwrap();
                    if let SerialAPIFrame::Command(ref cmd) = &msg {
                        // Send ACK
                        port_ref.write(ACK_BUFFER.to_vec()).unwrap();
                        if cmd.as_raw()[1] == 0x0b {
                            port_ref.write(hex::decode("01030002fe").unwrap()).unwrap();
                        }
                    }
                    println!("{:?}", msg);
                }
                thread::sleep(Duration::from_millis(100));
            }
        })
        .join()
        .unwrap();
    }

    println!("received 2 messages... closing");
    thread::sleep(Duration::from_millis(2000));

    let lock = Arc::try_unwrap(port_ref).expect("Lock still has multiple owners");
    let port = lock.into_inner().expect("Cannot lock mutex");
    port.close().unwrap();
}
