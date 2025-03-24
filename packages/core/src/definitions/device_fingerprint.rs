use super::{Id16, Version};

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceFingerprint {
    manufacturer_id: Id16,
    product_type: Id16,
    product_id: Id16,
    firmware_version: Version,
}

impl DeviceFingerprint {
    pub fn new(
        manufacturer_id: impl Into<Id16>,
        product_type: impl Into<Id16>,
        product_id: impl Into<Id16>,
        firmware_version: Version,
    ) -> Self {
        Self {
            manufacturer_id: manufacturer_id.into(),
            product_type: product_type.into(),
            product_id: product_id.into(),
            firmware_version,
        }
    }
}
