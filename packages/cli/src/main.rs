use crossbeam::thread;
use serial::common::*;
use serial::serial::SerialPortBinding;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let port = SerialPortBinding::new("/dev/ttyUSB0");

    let port = port.open().unwrap();

    {
        let listener = port.listener();
        let writer = port.writer();
        let writer2 = writer.clone();

        writer
            .write_raw(&hex::decode("01030008f4").unwrap())
            .unwrap();

        thread::scope(|s| {
            s.spawn(|_| {
                for frame in listener.iter().take(2) {
                    {
                        let data = frame.as_ref();
                        match &frame {
                            SerialAPIFrame::Command(_) => {
                                println!("<< {}", hex::encode(&data));
                            }
                            SerialAPIFrame::Garbage(_) => {
                                println!("DISCARDED: {}", hex::encode(&data));
                            }
                            SerialAPIFrame::ACK | SerialAPIFrame::CAN | SerialAPIFrame::NAK => {
                                println!("<< {:?}", &frame);
                            }
                        }

                        if let SerialAPIFrame::Command(_) = &frame {
                            // Send ACK
                            writer2.write(SerialAPIFrame::ACK).unwrap();
                            if data[1] == 0x0b {
                                writer2
                                    .write_raw(hex::decode("01030002fe").unwrap())
                                    .unwrap();
                            }
                        }
                        println!("received {:?}", frame);
                    }
                    sleep(Duration::from_millis(100));
                }
            });
        })
        .unwrap();

        println!("received 2 messages... closing");
        sleep(Duration::from_millis(2000));

        drop(writer)
    }

    port.close().unwrap();
}
