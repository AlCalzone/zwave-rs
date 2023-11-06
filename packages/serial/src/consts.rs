use derive_try_from_primitive::*;

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum SerialAPIControlByte {
    SOF = 0x01,
    ACK = 0x06,
    NAK = 0x15,
    CAN = 0x18,
}
