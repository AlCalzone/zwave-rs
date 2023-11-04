use crate::error::Result;

const SOF: u8 = 0x01;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const CAN: u8 = 0x18;

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SerialAPIControlByte {
    SOF = SOF as isize,
    ACK = ACK as isize,
    NAK = NAK as isize,
    CAN = CAN as isize,
}

#[derive(Debug)]
pub enum SerialAPIFrame {
    Control(SerialAPIControlByte),
    Command(Vec<u8>),
    Garbage(Vec<u8>),
}

fn contains_complete_command(buf: &[u8]) -> bool {
    return buf.len() >= 5 && buf.len() >= get_command_len(buf);
}

fn get_command_len(buf: &[u8]) -> usize {
    return (buf[1] + 2) as usize;
}

impl SerialAPIFrame {
    pub fn try_parse(buf: &[u8]) -> Option<(SerialAPIFrame, usize)> {
        if buf.len() == 0 {
            return None;
        }

        match buf[0] {
            ACK => return Some((SerialAPIFrame::Control(SerialAPIControlByte::ACK), 1)),
            CAN => return Some((SerialAPIFrame::Control(SerialAPIControlByte::CAN), 1)),
            NAK => return Some((SerialAPIFrame::Control(SerialAPIControlByte::NAK), 1)),
            SOF => {
                if contains_complete_command(buf) {
                    let cmd_len = get_command_len(buf);
                    return Some((SerialAPIFrame::Command(buf[..cmd_len].into()), cmd_len));
                } else {
                    // The buffer contains no complete message yet
                    return None;
                }
            }
            _ => {
                // INS12350: A host or a Z-Wave chip waiting for new traffic MUST ignore all other
                // byte values than 0x06 (ACK), 0x15 (NAK), 0x18 (CAN) or 0x01 (Data frame).

                // Find the position of the next valid byte
                let skip = buf
                    .iter()
                    .position(|&v| v == SOF || v == ACK || v == CAN || v == NAK)
                    // or discard the entire buffer
                    .unwrap_or(buf.len());
                return Some((SerialAPIFrame::Garbage(buf[..skip].into()), skip));
            }
        }
    }
}

pub type SerialAPIListener = crossbeam_channel::Sender<SerialAPIFrame>;

pub trait PortBinding {
    type Open;

    fn new(path: &str) -> Self;

    fn open(self, listener: SerialAPIListener) -> Result<Self::Open>;
}

pub trait OpenPortBinding {
    type Closed;

    fn close(self) -> Result<Self::Closed>;
    fn write(&mut self, data: Vec<u8>) -> Result<()>;
}
