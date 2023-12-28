use crate::prelude::*;
use ux::{u3, u5};
use zwave_core::{
    encoding::{encoders, BitParsable, BitSerializable, NomTryFromPrimitive},
    prelude::*,
};

use cookie_factory as cf;
use derive_try_from_primitive::TryFromPrimitive;
use nom::{
    bits, bits::complete::take as take_bits, bytes::complete::take, combinator::map_res,
    number::complete::be_u16, sequence::tuple,
};
use typed_builder::TypedBuilder;
use zwave_core::encoding::{self, encoders::empty};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ManufacturerSpecificCCCommand {
    Get = 0x04,
    Report = 0x05,
    DeviceSpecificGet = 0x06,
    DeviceSpecificReport = 0x07,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum DeviceIdType {
    FactoryDefault = 0x00,
    SerialNumber = 0x01,
    PseudoRandom = 0x02,
}

impl NomTryFromPrimitive for DeviceIdType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown device id type: {:#04x}", repr)
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct ManufacturerSpecificCCGet {}

impl CCBase for ManufacturerSpecificCCGet {}

impl CCId for ManufacturerSpecificCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::Get as _)
    }
}

impl CCRequest for ManufacturerSpecificCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::ManufacturerSpecificCCReport(_))
    }
}

impl CCParsable for ManufacturerSpecificCCGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CCSerializable for ManufacturerSpecificCCGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct ManufacturerSpecificCCReport {
    manufacturer_id: u16,
    product_type: u16,
    product_id: u16,
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
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, manufacturer_id) = be_u16(i)?;
        let (i, product_type) = be_u16(i)?;
        let (i, product_id) = be_u16(i)?;

        Ok((
            i,
            Self {
                manufacturer_id,
                product_type,
                product_id,
            },
        ))
    }
}

impl CCSerializable for ManufacturerSpecificCCReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u16, sequence::tuple};
        tuple((
            be_u16(self.manufacturer_id),
            be_u16(self.product_type),
            be_u16(self.product_id),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct ManufacturerSpecificCCDeviceSpecificGet {
    device_id_type: DeviceIdType,
}

impl CCBase for ManufacturerSpecificCCDeviceSpecificGet {}

impl CCId for ManufacturerSpecificCCDeviceSpecificGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::DeviceSpecificGet as _)
    }
}

impl CCRequest for ManufacturerSpecificCCDeviceSpecificGet {
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

impl CCParsable for ManufacturerSpecificCCDeviceSpecificGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, (_reserved73, device_id_type)) = bits(tuple((
            u5::parse,
            map_res(take_bits(3usize), |x: u8| {
                DeviceIdType::try_from_primitive(x)
            }),
        )))(i)?;

        Ok((i, Self { device_id_type }))
    }
}

impl CCSerializable for ManufacturerSpecificCCDeviceSpecificGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        encoders::bits(move |bo| {
            u5::new(0).write(bo);
            u3::new(((self.device_id_type) as u8) & 0b0000_0111).write(bo);
        })
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct ManufacturerSpecificCCDeviceSpecificReport {
    device_id_type: DeviceIdType,
    device_id: Vec<u8>, // FIXME: Actually string or buffer
}

impl CCBase for ManufacturerSpecificCCDeviceSpecificReport {}

impl CCId for ManufacturerSpecificCCDeviceSpecificReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::ManufacturerSpecific
    }

    fn cc_command(&self) -> Option<u8> {
        Some(ManufacturerSpecificCCCommand::DeviceSpecificReport as _)
    }
}

impl CCParsable for ManufacturerSpecificCCDeviceSpecificReport {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, (_reserved73, device_id_type)) = bits(tuple((
            u5::parse,
            map_res(take_bits(3usize), |x: u8| {
                DeviceIdType::try_from_primitive(x)
            }),
        )))(i)?;
        let (i, (_data_format, data_len)) = bits(tuple((u3::parse, u5::parse)))(i)?;
        let (i, device_id) = take(u8::from(data_len))(i)?;

        Ok((
            i,
            Self {
                device_id_type,
                device_id: device_id.to_vec(),
            },
        ))
    }
}

impl CCSerializable for ManufacturerSpecificCCDeviceSpecificReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};
        move |out| {
            todo!("ERROR: ManufacturerSpecificCCDeviceSpecificReport::serialize() not implemented")
        }
    }
}
