use crc16::*;

/// Compute the XOR "checksum" of the given data
pub fn xor_sum(data: &[u8]) -> u8 {
    data.iter().fold(0xff, |acc, x| acc ^ x)
}

/// Computes the CRC16 checksum of the given data
pub fn crc16(data: &[u8]) -> u16 {
    State::<AUG_CCITT>::calculate(data)
}

pub struct Crc16(State<AUG_CCITT>);

impl Crc16 {
    pub fn update(mut self, data: &[u8]) -> Self {
        self.0.update(data);
        self
    }

    pub fn get(&self) -> u16 {
        self.0.get()
    }
}

pub fn crc16_incremental() -> Crc16 {
    Crc16(State::<AUG_CCITT>::new())
}

#[test]
fn test_xor_sum() {
    let input = hex::decode("030002").unwrap();
    let expected = 0xfe;
    assert_eq!(xor_sum(&input), expected);
}

#[test]
fn test_crc16() {
    assert_eq!(crc16(&[]), 0x1d0f);
    assert_eq!(crc16(b"A"), 0x9479);
    assert_eq!(crc16(b"123456789"), 0xe5cc);
}

#[test]
fn test_crc16_incremental() {
    let mut crc = crc16_incremental();
    let input = b"123456789";
    for i in input {
        crc = crc.update(&[*i]);
    }
    assert_eq!(crc.get(), 0xe5cc);
}
