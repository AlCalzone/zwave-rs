use crate::encoding;
use crate::prelude::*;
use cookie_factory as cf;
use nom::{combinator::map_res, error::context, number::complete::be_u8};
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum ZWaveLibraryType {
    Unknown,
    StaticController,
    Controller,
    EnhancedSlave,
    Slave,
    Installer,
    RoutingSlave,
    BridgeController,
    DeviceUnderTest,
    NotApplicable,
    AvRemote,
    AvDevice,
}

impl Display for ZWaveLibraryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZWaveLibraryType::Unknown => write!(f, "Unknown"),
            ZWaveLibraryType::StaticController => write!(f, "Static Controller"),
            ZWaveLibraryType::Controller => write!(f, "Controller"),
            ZWaveLibraryType::EnhancedSlave => write!(f, "Enhanced Slave"),
            ZWaveLibraryType::Slave => write!(f, "Slave"),
            ZWaveLibraryType::Installer => write!(f, "Installer"),
            ZWaveLibraryType::RoutingSlave => write!(f, "Routing Slave"),
            ZWaveLibraryType::BridgeController => write!(f, "Bridge Controller"),
            ZWaveLibraryType::DeviceUnderTest => write!(f, "Device under Test"),
            ZWaveLibraryType::NotApplicable => write!(f, "N/A"),
            ZWaveLibraryType::AvRemote => write!(f, "AV Remote"),
            ZWaveLibraryType::AvDevice => write!(f, "AV Device"),
        }
    }
}

impl NomTryFromPrimitive for ZWaveLibraryType {
    type Repr = u8;

    fn format_error(repr: Self::Repr) -> String {
        format!("Unknown Z-Wave library type: {:#04x}", repr)
    }
}

impl Parsable for ZWaveLibraryType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ZWaveLibraryType",
            map_res(be_u8, ZWaveLibraryType::try_from_primitive),
        )(i)
    }
}

impl Serializable for ZWaveLibraryType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
