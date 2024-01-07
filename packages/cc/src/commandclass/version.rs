use crate::prelude::*;
use crate::values::*;
use cookie_factory as cf;
use nom::{
    combinator::{map, opt},
    multi::length_count,
    number::complete::{be_u16, be_u8},
    sequence::tuple,
};
use proc_macros::TryFromRepr;
use std::borrow::Cow;
use typed_builder::TypedBuilder;
use zwave_core::cache::CacheValue;
use zwave_core::encoding::{self, encoders::empty, parsers};
use zwave_core::prelude::*;
use zwave_core::util::ToDiscriminant;
use zwave_core::value_id::{ValueId, ValueIdProperties};

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)] // must match the ToDiscriminant impl
enum VersionCCProperties {
    FirmwareVersion(u8) = 0x00,
    LibraryType = 0x01,
    ProtocolVersion = 0x02,
    HardwareVersion = 0x03,
    SupportsZWaveSoftwareGet = 0x04,
    SDKVersion = 0x05,
    ApplicationFrameworkAPIVersion = 0x06,
    ApplicationFrameworkBuildNumber = 0x07,
    SerialAPIVersion = 0x08,
    SerialAPIBuildNumber = 0x09,
    ZWaveProtocolVersion = 0x0A,
    ZWaveProtocolBuildNumber = 0x0B,
    ApplicationVersion = 0x0C,
    ApplicationBuildNumber = 0x0D,
}

unsafe impl ToDiscriminant<u8> for VersionCCProperties {}

impl From<VersionCCProperties> for ValueIdProperties {
    fn from(val: VersionCCProperties) -> Self {
        match val {
            VersionCCProperties::FirmwareVersion(index) => {
                Self::new(val.to_discriminant(), Some(index as u32))
            }
            _ => Self::new(val.to_discriminant(), None),
        }
    }
}

impl TryFrom<ValueIdProperties> for VersionCCProperties {
    type Error = ();

    fn try_from(value: ValueIdProperties) -> Result<Self, Self::Error> {
        match (Self::try_from(value.property() as u8), value.property_key()) {
            // Static properties have no property key
            (Ok(prop), None) => return Ok(prop),
            // Dynamic properties have one
            (Err(TryFromReprError::NonPrimitive(d)), Some(k)) => {
                // Figure out which one it is
                let firmware_version_discr = Self::FirmwareVersion(0).to_discriminant();
                if d == firmware_version_discr && k <= u8::MAX as u32 {
                    return Ok(Self::FirmwareVersion(k as u8));
                }
            }
            _ => (),
        }

        Err(())
    }
}

pub struct VersionCCValues;
impl VersionCCValues {
    cc_value_dynamic_property!(
        Version,
        FirmwareVersion,
        |chip_index: u8| ValueMetadata::String(
            ValueMetadataString::default()
                .label(if chip_index == 0 {
                    Cow::from("Z-Wave chip firmware version")
                } else {
                    Cow::from(format!("Firmware version (chip #{})", chip_index))
                })
                .readonly()
        ),
        CCValueOptions::default().supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        LibraryType,
        ValueMetadata::Numeric(
            // FIXME: This should be limited to the ZWaveLibraryType enum range and states
            ValueMetadataNumeric::default()
                .label("Library type")
                .readonly()
        ),
        CCValueOptions::default().supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        ProtocolVersion,
        ValueMetadata::String(
            ValueMetadataString::default()
                .label("Z-Wave protocol version")
                .readonly()
        ),
        CCValueOptions::default().supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        HardwareVersion,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Z-Wave chip hardware version")
        ),
        CCValueOptions::default()
            .min_version(2)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        supports_zwave_software_get,
        SupportsZWaveSoftwareGet,
        ValueMetadata::Boolean(ValueMetadataBoolean::default().readonly()),
        CCValueOptions::default().min_version(3).internal()
    );

    cc_value_static_property!(
        Version,
        sdk_version,
        SDKVersion,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("SDK version")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        application_framework_api_version,
        ApplicationFrameworkAPIVersion,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Z-Wave application framework API version")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        ApplicationFrameworkBuildNumber,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Z-Wave application framework API build number")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        serial_api_version,
        SerialAPIVersion,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Serial API version")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        serial_api_build_number,
        SerialAPIBuildNumber,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Serial API build number")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        zwave_protocol_version,
        ZWaveProtocolVersion,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Z-Wave protocol version")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        zwave_protocol_build_number,
        ZWaveProtocolBuildNumber,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Z-Wave protocol build number")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        ApplicationVersion,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Application version")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );

    cc_value_static_property!(
        Version,
        ApplicationBuildNumber,
        ValueMetadata::Numeric(
            ValueMetadataNumeric::default()
                .readonly()
                .label("Application build number")
        ),
        CCValueOptions::default()
            .min_version(3)
            .supports_endpoints(false)
    );
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum VersionCCCommand {
    Get = 0x11,
    Report = 0x12,
    CommandClassGet = 0x13,
    CommandClassReport = 0x14,
    CapabilitiesGet = 0x15,
    CapabilitiesReport = 0x16,
    ZWaveSoftwareGet = 0x17,
    ZWaveSoftwareReport = 0x18,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VersionCCGet {}

impl CCBase for VersionCCGet {}

impl CCValues for VersionCCGet {}

impl CCId for VersionCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::Get as _)
    }
}

impl CCRequest for VersionCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::VersionCCReport(_))
    }
}

impl CCParsable for VersionCCGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CCSerializable for VersionCCGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct VersionCCReport {
    pub library_type: ZWaveLibraryType,
    pub protocol_version: Version,
    pub firmware_versions: Vec<Version>,
    pub hardware_version: Option<u8>,
}

impl CCBase for VersionCCReport {}

impl CCValues for VersionCCReport {
    fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
        let mut ret = vec![
            (
                VersionCCValues::library_type().id,
                CacheValue::from(self.library_type as u8),
            ),
            (
                VersionCCValues::protocol_version().id,
                CacheValue::from(self.protocol_version.to_string()),
            ),
        ];

        ret.extend(
            self.firmware_versions
                .iter()
                .enumerate()
                .map(|(i, version)| {
                    (
                        VersionCCValues::firmware_version().eval((i as u8,)).id,
                        CacheValue::from(version.to_string()),
                    )
                }),
        );

        if let Some(hardware_version) = self.hardware_version {
            ret.push((
                VersionCCValues::hardware_version().id,
                CacheValue::from(hardware_version),
            ));
        }

        ret
    }
}

impl CCId for VersionCCReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::Report as _)
    }
}

impl CCParsable for VersionCCReport {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, library_type) = ZWaveLibraryType::parse(i)?;
        let (i, protocol_version) = parsers::version_major_minor(i)?;
        let (i, firmware_0_version) = parsers::version_major_minor(i)?;
        let (i, (hardware_version, additional_firmware_versions)) = map(
            opt(tuple((
                be_u8,
                length_count(be_u8, parsers::version_major_minor),
            ))),
            Option::unzip,
        )(i)?;
        let firmware_versions = {
            let mut versions = vec![firmware_0_version];
            versions.extend(additional_firmware_versions.unwrap_or_default());
            versions
        };

        Ok((
            i,
            Self {
                library_type,
                protocol_version,
                firmware_versions,
                hardware_version,
            },
        ))
    }
}

impl CCSerializable for VersionCCReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        // use cf::{bytes::be_u8, sequence::tuple};
        move |_out| todo!("ERROR: VersionCCReport::serialize() not implemented")
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct VersionCCCommandClassGet {
    requested_cc: CommandClasses,
}

impl CCBase for VersionCCCommandClassGet {}

impl CCValues for VersionCCCommandClassGet {}

impl CCId for VersionCCCommandClassGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CommandClassGet as _)
    }
}

impl CCRequest for VersionCCCommandClassGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match response {
            CC::VersionCCCommandClassReport(VersionCCCommandClassReport {
                requested_cc, ..
            }) if (requested_cc == &self.requested_cc) => true,
            _ => false,
        }
    }
}

impl CCParsable for VersionCCCommandClassGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, requested_cc) = CommandClasses::parse(i)?;

        Ok((i, Self { requested_cc }))
    }
}

impl CCSerializable for VersionCCCommandClassGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        self.requested_cc.serialize()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct VersionCCCommandClassReport {
    pub requested_cc: CommandClasses,
    pub version: u8,
}

impl CCBase for VersionCCCommandClassReport {}

impl CCValues for VersionCCCommandClassReport {}

impl CCId for VersionCCCommandClassReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CommandClassReport as _)
    }
}

impl CCParsable for VersionCCCommandClassReport {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, requested_cc) = CommandClasses::parse(i)?;
        let (i, version) = be_u8(i)?;

        Ok((
            i,
            Self {
                requested_cc,
                version,
            },
        ))
    }
}

impl CCSerializable for VersionCCCommandClassReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};

        tuple((self.requested_cc.serialize(), be_u8(self.version)))
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VersionCCCapabilitiesGet {}

impl CCBase for VersionCCCapabilitiesGet {}

impl CCValues for VersionCCCapabilitiesGet {}

impl CCId for VersionCCCapabilitiesGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CapabilitiesGet as _)
    }
}

impl CCRequest for VersionCCCapabilitiesGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::VersionCCCapabilitiesReport(_))
    }
}

impl CCParsable for VersionCCCapabilitiesGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CCSerializable for VersionCCCapabilitiesGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct VersionCCCapabilitiesReport {
    pub supports_zwave_software_get: bool,
}

impl CCBase for VersionCCCapabilitiesReport {}

impl CCValues for VersionCCCapabilitiesReport {
    fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
        vec![(
            VersionCCValues::supports_zwave_software_get().id,
            CacheValue::from(self.supports_zwave_software_get),
        )]
    }
}

impl CCId for VersionCCCapabilitiesReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CapabilitiesReport as _)
    }
}

impl CCParsable for VersionCCCapabilitiesReport {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, capabilities) = be_u8(i)?;
        let supports_zwave_software_get = capabilities & 0b100 != 0;

        Ok((
            i,
            Self {
                supports_zwave_software_get,
            },
        ))
    }
}

impl CCSerializable for VersionCCCapabilitiesReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        let capabilities = if self.supports_zwave_software_get {
            0b100
        } else {
            0
        };
        be_u8(capabilities)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VersionCCZWaveSoftwareGet {}

impl CCBase for VersionCCZWaveSoftwareGet {}

impl CCValues for VersionCCZWaveSoftwareGet {}

impl CCId for VersionCCZWaveSoftwareGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::ZWaveSoftwareGet as _)
    }
}

impl CCRequest for VersionCCZWaveSoftwareGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::VersionCCZWaveSoftwareReport(_))
    }
}

impl CCParsable for VersionCCZWaveSoftwareGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CCSerializable for VersionCCZWaveSoftwareGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct VersionCCZWaveSoftwareReport {
    sdk_version: Version,
    application_framework_version: Option<(Version, u16)>,
    host_interface_version: Option<(Version, u16)>,
    zwave_protocol_version: Option<(Version, u16)>,
    application_version: Option<(Version, u16)>,
}

impl CCBase for VersionCCZWaveSoftwareReport {}

impl CCValues for VersionCCZWaveSoftwareReport {
    fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
        let mut ret = vec![(
            // FIXME: we should have an override for the name
            VersionCCValues::sdk_version().id,
            CacheValue::from(self.sdk_version.to_string()),
        )];

        if let Some((v, b)) = self.application_framework_version {
            ret.push((
                VersionCCValues::application_framework_api_version().id,
                CacheValue::from(v.to_string()),
            ));
            ret.push((
                VersionCCValues::application_framework_build_number().id,
                CacheValue::from(b),
            ));
        }

        if let Some((v, b)) = self.host_interface_version {
            ret.push((
                VersionCCValues::serial_api_version().id,
                CacheValue::from(v.to_string()),
            ));
            ret.push((
                VersionCCValues::serial_api_build_number().id,
                CacheValue::from(b),
            ));
        }

        if let Some((v, b)) = self.zwave_protocol_version {
            ret.push((
                VersionCCValues::zwave_protocol_version().id,
                CacheValue::from(v.to_string()),
            ));
            ret.push((
                VersionCCValues::zwave_protocol_build_number().id,
                CacheValue::from(b),
            ));
        }

        if let Some((v, b)) = self.application_version {
            ret.push((
                VersionCCValues::application_version().id,
                CacheValue::from(v.to_string()),
            ));
            ret.push((
                VersionCCValues::application_build_number().id,
                CacheValue::from(b),
            ));
        }

        ret
    }
}

impl CCId for VersionCCZWaveSoftwareReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::ZWaveSoftwareReport as _)
    }
}

impl CCParsable for VersionCCZWaveSoftwareReport {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        fn parse_opt_version_and_build_number(
            i: encoding::Input,
        ) -> encoding::ParseResult<Option<(Version, u16)>> {
            map(
                tuple((parsers::version_major_minor_patch, be_u16)),
                |(version, build_number)| {
                    if version.major == 0 && version.minor == 0 && version.patch == Some(0) {
                        None
                    } else {
                        Some((version, build_number))
                    }
                },
            )(i)
        }

        let (i, sdk_version) = parsers::version_major_minor_patch(i)?;
        let (i, application_framework_version) = parse_opt_version_and_build_number(i)?;
        let (i, host_interface_version) = parse_opt_version_and_build_number(i)?;
        let (i, zwave_protocol_version) = parse_opt_version_and_build_number(i)?;
        let (i, application_version) = parse_opt_version_and_build_number(i)?;

        Ok((
            i,
            Self {
                sdk_version,
                application_framework_version,
                host_interface_version,
                zwave_protocol_version,
                application_version,
            },
        ))
    }
}

impl CCSerializable for VersionCCZWaveSoftwareReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        move |_out| todo!("ERROR: VersionCCZWaveSoftwareReport::serialize() not implemented")
    }
}
