use crate::prelude::*;
use zwave_core::{encoding::parsers, prelude::*};

use cookie_factory as cf;
use derive_try_from_primitive::TryFromPrimitive;
use nom::{
    combinator::{map, opt},
    multi::length_count,
    number::complete::{be_u16, be_u8},
    sequence::tuple,
};
use typed_builder::TypedBuilder;
use zwave_core::encoding::{self, encoders::empty};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
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

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
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
    library_type: ZWaveLibraryType,
    protocol_version: Version,
    firmware_versions: Vec<Version>,
    hardware_version: Option<u8>,
}

impl CCBase for VersionCCReport {}

impl CCValues for VersionCCReport {}

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
        move |out| todo!("ERROR: VersionCCReport::serialize() not implemented")
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
    requested_cc: CommandClasses,
    version: u8,
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

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
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
    supports_zwave_software_get: bool,
}

impl CCBase for VersionCCCapabilitiesReport {}

impl CCValues for VersionCCCapabilitiesReport {}

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

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
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

impl CCValues for VersionCCZWaveSoftwareReport {}

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
        use cf::{bytes::be_u8, sequence::tuple};
        move |out| todo!("ERROR: VersionCCZWaveSoftwareReport::serialize() not implemented")
    }
}
