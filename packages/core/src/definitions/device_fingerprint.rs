use custom_debug_derive::Debug;
use super::Version;

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceFingerprint {
    #[debug(format = "0x{:04x}")]
    manufacturer_id: u16,
    #[debug(format = "0x{:04x}")]
    product_type: u16,
    #[debug(format = "0x{:04x}")]
    product_id: u16,
    firmware_version: Version,
}

impl DeviceFingerprint {
    pub fn new(
        manufacturer_id: u16,
        product_type: u16,
        product_id: u16,
        firmware_version: Version,
    ) -> Self {
        Self {
            manufacturer_id,
            product_type,
            product_id,
            firmware_version,
        }
    }
}
