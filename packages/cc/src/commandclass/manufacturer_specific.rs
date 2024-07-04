use crate::prelude::*;
use crate::values::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use std::fmt::Display;
use typed_builder::TypedBuilder;
use ux::{u3, u5};
use zwave_core::cache::CacheValue;
use zwave_core::parse::{
    bits,
    bytes::{be_u16, complete::take},
    combinators::map_res,
};
use zwave_core::prelude::*;
use zwave_core::serialize;
use zwave_core::util::ToDiscriminant;
use zwave_core::value_id::{ValueId, ValueIdProperties};

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)] // must match the ToDiscriminant impl
enum ManufacturerSpecificCCProperties {
    ManufacturerId = 0x00,
    ProductType = 0x01,
    ProductId = 0x02,
    DeviceId(DeviceIdType) = 0x03,
}

unsafe impl ToDiscriminant<u8> for ManufacturerSpecificCCProperties {}

impl From<ManufacturerSpecificCCProperties> for ValueIdProperties {
    fn from(val: ManufacturerSpecificCCProperties) -> Self {
        match val {
            ManufacturerSpecificCCProperties::DeviceId(device_id_type) => {
                Self::new(val.to_discriminant(), Some(device_id_type as u32))
            }
            _ => Self::new(val.to_discriminant(), None),
        }
    }
}

impl TryFrom<ValueIdProperties> for ManufacturerSpecificCCProperties {
    type Error = ();

    fn try_from(value: ValueIdProperties) -> Result<Self, Self::Error> {
        match (Self::try_from(value.property() as u8), value.property_key()) {
            // Static properties have no property key
            (Ok(prop), None) => return Ok(prop),
            // Dynamic properties have one
            (Err(TryFromReprError::NonPrimitive(d)), Some(k)) => {
                // Figure out which one it is
                let device_id_discr =
                    Self::DeviceId(DeviceIdType::FactoryDefault).to_discriminant();
                if d == device_id_discr {
                    let Ok(device_id) = DeviceIdType::try_from(k as u8) else {
                        return Err(());
                    };
                    return Ok(Self::DeviceId(device_id));
                }
            }
            _ => (),
        }

        Err(())
    }
}

#[test]
fn test_device_id_value() {
    let value = ManufacturerSpecificCCValues::device_id();
    let value_id = ValueId::new(
        CommandClasses::ManufacturerSpecific,
        0x03u32,
        Some(DeviceIdType::SerialNumber as u32),
    );
    assert!(value.is(&value_id));
    assert!(!value.options.supports_endpoints);

    let evaluated = value.eval((DeviceIdType::SerialNumber,));
    assert_eq!(evaluated.id, value_id);
    match evaluated.metadata {
        ValueMetadata::Buffer(meta) => {
            assert_eq!(meta.common.label.unwrap(), "Device ID (serial number)");
            assert!(meta.common.readable);
            assert!(!meta.common.writeable);
        }
        _ => panic!("Unexpected metadata: {:?}", evaluated.metadata),
    }
}

pub struct ManufacturerSpecificCCValues;
impl ManufacturerSpecificCCValues {
    cc_value_static_property!(
        ManufacturerSpecific,
        ManufacturerId,
        ValueMetadata::Numeric(ValueMetadataNumeric::readonly_u16().label("Manufacturer ID")),
        CCValueOptions::default().supports_endpoints(false)
    );

    cc_value_static_property!(
        ManufacturerSpecific,
        ProductType,
        ValueMetadata::Numeric(ValueMetadataNumeric::readonly_u16().label("Product type")),
        CCValueOptions::default().supports_endpoints(false)
    );

    cc_value_static_property!(
        ManufacturerSpecific,
        ProductId,
        ValueMetadata::Numeric(ValueMetadataNumeric::readonly_u16().label("Product ID")),
        CCValueOptions::default().supports_endpoints(false)
    );

    cc_value_dynamic_property!(
        ManufacturerSpecific,
        DeviceId,
        |device_id_type: DeviceIdType| ValueMetadata::Buffer(
            ValueMetadataBuffer::default()
                .readonly()
                .label(format!("Device ID ({})", device_id_type))
        ),
        CCValueOptions::default()
            .supports_endpoints(false)
            .min_version(2)
    );
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum ManufacturerSpecificCCCommand {
    Get = 0x04,
    Report = 0x05,
    DeviceSpecificGet = 0x06,
    DeviceSpecificReport = 0x07,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum DeviceIdType {
    FactoryDefault = 0x00,
    SerialNumber = 0x01,
    PseudoRandom = 0x02,
}

impl Display for DeviceIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceIdType::FactoryDefault => write!(f, "factory default"),
            DeviceIdType::SerialNumber => write!(f, "serial number"),
            DeviceIdType::PseudoRandom => write!(f, "pseudo-random"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct ManufacturerSpecificCCGet {}

impl CCBase for ManufacturerSpecificCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::ManufacturerSpecificCCReport(_))
    }
}

impl CCId for ManufacturerSpecificCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::Get as _)
    }
}

impl CCParsable for ManufacturerSpecificCCGet {
    fn parse(_i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for ManufacturerSpecificCCGet {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for ManufacturerSpecificCCGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct ManufacturerSpecificCCReport {
    #[cc_value(ManufacturerSpecificCCValues::manufacturer_id)]
    pub manufacturer_id: u16,
    #[cc_value(ManufacturerSpecificCCValues::product_type)]
    pub product_type: u16,
    #[cc_value(ManufacturerSpecificCCValues::product_id)]
    pub product_id: u16,
}

impl CCBase for ManufacturerSpecificCCReport {}

impl CCId for ManufacturerSpecificCCReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::Report as _)
    }
}

impl CCParsable for ManufacturerSpecificCCReport {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let manufacturer_id = be_u16(i)?;
        let product_type = be_u16(i)?;
        let product_id = be_u16(i)?;

        Ok(Self {
            manufacturer_id,
            product_type,
            product_id,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for ManufacturerSpecificCCReport {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::{bytes::be_u16, sequence::tuple};
        tuple((
            be_u16(self.manufacturer_id),
            be_u16(self.product_type),
            be_u16(self.product_id),
        ))
        .serialize(output)
    }
}

impl ToLogPayload for ManufacturerSpecificCCReport {
    fn to_log_payload(&self) -> LogPayload {
        let ret = LogPayloadDict::new()
            .with_entry("manufacturer ID", format!("0x{:04x}", self.manufacturer_id))
            .with_entry("product type", format!("0x{:04x}", self.product_type))
            .with_entry("product ID", format!("0x{:04x}", self.product_id));

        ret.into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct ManufacturerSpecificCCDeviceSpecificGet {
    device_id_type: DeviceIdType,
}

impl CCBase for ManufacturerSpecificCCDeviceSpecificGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(
            response,
            CC::ManufacturerSpecificCCDeviceSpecificReport(
                ManufacturerSpecificCCDeviceSpecificReport {
                    device_id_type,
                    ..
                }
            ) if device_id_type == &self.device_id_type
        )
    }
}

impl CCId for ManufacturerSpecificCCDeviceSpecificGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::DeviceSpecificGet as _)
    }
}

impl CCParsable for ManufacturerSpecificCCDeviceSpecificGet {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let (_reserved73, device_id_type) = bits::bits((
            u5::parse,
            map_res(bits::take(3usize), |x: u8| DeviceIdType::try_from(x)),
        ))
        .parse(i)?;

        Ok(Self { device_id_type })
    }
}

impl SerializableWith<&CCEncodingContext> for ManufacturerSpecificCCDeviceSpecificGet {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::bits::bits;
        bits(move |bo| {
            u5::new(0).write(bo);
            u3::new(((self.device_id_type) as u8) & 0b0000_0111).write(bo);
        })
        .serialize(output)
    }
}

impl ToLogPayload for ManufacturerSpecificCCDeviceSpecificGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("device ID type", self.device_id_type.to_string())
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct ManufacturerSpecificCCDeviceSpecificReport {
    // FIXME: Extend the CCValues derive macro to support dynamic values with cross-references
    pub device_id_type: DeviceIdType,
    pub device_id: Vec<u8>, // FIXME: Actually string or buffer
}

impl CCBase for ManufacturerSpecificCCDeviceSpecificReport {}

impl CCValues for ManufacturerSpecificCCDeviceSpecificReport {
    fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
        vec![(
            ManufacturerSpecificCCValues::device_id()
                .eval((self.device_id_type,))
                .id,
            CacheValue::from(self.device_id.clone()),
        )]
    }
}

impl CCId for ManufacturerSpecificCCDeviceSpecificReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::DeviceSpecificReport as _)
    }
}

impl CCParsable for ManufacturerSpecificCCDeviceSpecificReport {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let (_reserved73, device_id_type) = bits::bits((
            u5::parse,
            map_res(bits::take(3usize), |x: u8| DeviceIdType::try_from(x)),
        ))
        .parse(i)?;
        let (_data_format, data_len) = bits::bits((u3::parse, u5::parse)).parse(i)?;
        let device_id = take(u8::from(data_len)).parse(i)?;

        Ok(Self {
            device_id_type,
            device_id: device_id.to_vec(),
        })
    }
}

impl SerializableWith<&CCEncodingContext> for ManufacturerSpecificCCDeviceSpecificReport {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        todo!("ERROR: ManufacturerSpecificCCDeviceSpecificReport::serialize() not implemented")
    }
}

impl ToLogPayload for ManufacturerSpecificCCDeviceSpecificReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("device ID type", self.device_id_type.to_string())
            .with_entry("device ID", format!("0x{}", hex::encode(&self.device_id)))
            .into()
    }
}
