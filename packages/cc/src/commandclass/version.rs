use crate::prelude::*;
use crate::values::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use std::borrow::Cow;
use typed_builder::TypedBuilder;
use zwave_core::cache::CacheValue;
use zwave_core::parse::{
    bytes::{be_u16, be_u8},
    combinators::{map, map_repeat, opt},
};
use zwave_core::prelude::*;
use zwave_core::serialize::{self, Serializable};
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

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct VersionCCGet {}

impl CCBase for VersionCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::VersionCCReport(_))
    }
}

impl CCId for VersionCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::Get as _)
    }
}

impl CCParsable for VersionCCGet {
    fn parse(_i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCGet {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for VersionCCGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
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
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let library_type = ZWaveLibraryType::parse(i)?;
        let protocol_version = Version::parse_major_minor(i)?;
        let firmware_0_version = Version::parse_major_minor(i)?;
        let (hardware_version, additional_firmware_versions) = map(
            opt((be_u8, map_repeat(be_u8, Version::parse_major_minor))),
            Option::unzip,
        )
        .parse(i)?;
        let firmware_versions = {
            let mut versions = vec![firmware_0_version];
            versions.extend(additional_firmware_versions.unwrap_or_default());
            versions
        };

        Ok(Self {
            library_type,
            protocol_version,
            firmware_versions,
            hardware_version,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCReport {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        todo!("ERROR: VersionCCReport::serialize() not implemented")
    }
}

impl ToLogPayload for VersionCCReport {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new()
            .with_entry("library type", self.library_type.to_string())
            .with_entry("protocol version", self.protocol_version.to_string())
            .with_entry(
                "firmware versions",
                self.firmware_versions
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        if let Some(hardware_version) = self.hardware_version {
            ret = ret.with_entry("hardware version", hardware_version);
        }

        ret.into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct VersionCCCommandClassGet {
    requested_cc: CommandClasses,
}

impl CCBase for VersionCCCommandClassGet {
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

impl CCId for VersionCCCommandClassGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CommandClassGet as _)
    }
}

impl CCParsable for VersionCCCommandClassGet {
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let requested_cc = CommandClasses::parse(i)?;

        Ok(Self { requested_cc })
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCCommandClassGet {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        self.requested_cc.serialize(output);
    }
}

impl ToLogPayload for VersionCCCommandClassGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("requested CC", self.requested_cc.to_string())
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct VersionCCCommandClassReport {
    pub requested_cc: CommandClasses,
    pub version: u8,
}

impl CCBase for VersionCCCommandClassReport {}

impl CCId for VersionCCCommandClassReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CommandClassReport as _)
    }
}

impl CCParsable for VersionCCCommandClassReport {
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let requested_cc = CommandClasses::parse(i)?;
        let version = be_u8(i)?;

        Ok(Self {
            requested_cc,
            version,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCCommandClassReport {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::bytes::be_u8;
        self.requested_cc.serialize(output);
        be_u8(self.version).serialize(output);
    }
}

impl ToLogPayload for VersionCCCommandClassReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("requested CC", self.requested_cc.to_string())
            .with_entry("version", self.version)
            .into()
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct VersionCCCapabilitiesGet {}

impl CCBase for VersionCCCapabilitiesGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::VersionCCCapabilitiesReport(_))
    }
}

impl CCId for VersionCCCapabilitiesGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CapabilitiesGet as _)
    }
}

impl CCParsable for VersionCCCapabilitiesGet {
    fn parse(_i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCCapabilitiesGet {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for VersionCCCapabilitiesGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct VersionCCCapabilitiesReport {
    #[cc_value(VersionCCValues::supports_zwave_software_get)]
    pub supports_zwave_software_get: bool,
}

impl CCBase for VersionCCCapabilitiesReport {}

impl CCId for VersionCCCapabilitiesReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::CapabilitiesReport as _)
    }
}

impl CCParsable for VersionCCCapabilitiesReport {
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let capabilities = be_u8(i)?;
        let supports_zwave_software_get = capabilities & 0b100 != 0;

        Ok(Self {
            supports_zwave_software_get,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCCapabilitiesReport {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::bytes::be_u8;
        let capabilities = if self.supports_zwave_software_get {
            0b100
        } else {
            0
        };
        be_u8(capabilities).serialize(output);
    }
}

impl ToLogPayload for VersionCCCapabilitiesReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry(
                "supports Z-Wave Software Get",
                self.supports_zwave_software_get,
            )
            .into()
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct VersionCCZWaveSoftwareGet {}

impl CCBase for VersionCCZWaveSoftwareGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::VersionCCZWaveSoftwareReport(_))
    }
}

impl CCId for VersionCCZWaveSoftwareGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_command(&self) -> Option<u8> {
        Some(VersionCCCommand::ZWaveSoftwareGet as _)
    }
}

impl CCParsable for VersionCCZWaveSoftwareGet {
    fn parse(_i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCZWaveSoftwareGet {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for VersionCCZWaveSoftwareGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
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
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        fn parse_opt_version_and_build_number(
            i: &mut Bytes,
        ) -> zwave_core::parse::ParseResult<Option<(Version, u16)>> {
            map(
                (Version::parse_major_minor_patch, be_u16),
                |(version, build_number)| {
                    if version.major == 0 && version.minor == 0 && version.patch == Some(0) {
                        None
                    } else {
                        Some((version, build_number))
                    }
                },
            )
            .parse(i)
        }

        let sdk_version = Version::parse_major_minor_patch(i)?;
        let application_framework_version = parse_opt_version_and_build_number(i)?;
        let host_interface_version = parse_opt_version_and_build_number(i)?;
        let zwave_protocol_version = parse_opt_version_and_build_number(i)?;
        let application_version = parse_opt_version_and_build_number(i)?;

        Ok(Self {
            sdk_version,
            application_framework_version,
            host_interface_version,
            zwave_protocol_version,
            application_version,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for VersionCCZWaveSoftwareReport {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        todo!("ERROR: VersionCCZWaveSoftwareReport::serialize() not implemented")
    }
}

impl ToLogPayload for VersionCCZWaveSoftwareReport {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new().with_entry("SDK version", self.sdk_version.to_string());
        if let Some((v, b)) = self.application_framework_version {
            ret = ret
                .with_entry("application framework version", v.to_string())
                .with_entry("application framework build number", b);
        }
        if let Some((v, b)) = self.host_interface_version {
            ret = ret
                .with_entry("host interface version", v.to_string())
                .with_entry("host interface build number", b);
        }
        if let Some((v, b)) = self.zwave_protocol_version {
            ret = ret
                .with_entry("Z-Wave protocol version", v.to_string())
                .with_entry("Z-Wave protocol build number", b);
        }
        if let Some((v, b)) = self.application_version {
            ret = ret
                .with_entry("application version", v.to_string())
                .with_entry("application build number", b);
        }

        ret.into()
    }
}
