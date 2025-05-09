use crate::parse::{
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::{BytesMut, Bytes};
use crate::serialize::{self, Serializable};
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

impl Parsable for ZWaveLibraryType {
    fn parse(i: &mut Bytes) -> crate::parse::ParseResult<Self> {
        context("ZWaveLibraryType", map_res(be_u8, Self::try_from)).parse(i)
    }
}

impl Serializable for ZWaveLibraryType {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output)
    }
}
