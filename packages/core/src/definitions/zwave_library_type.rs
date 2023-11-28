use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ZWaveLibraryType {
    Unknown,
    #[debug(format = "Static Controller")]
    StaticController,
    Controller,
    #[debug(format = "Enhanced Slave")]
    EnhancedSlave,
    Slave,
    Installer,
    #[debug(format = "Routing Slave")]
    RoutingSlave,
    #[debug(format = "Bridge Controller")]
    BridgeController,
    #[debug(format = "Device under Test")]
    DeviceUnderTest,
    #[debug(format = "N/A")]
    NotApplicable,
    #[debug(format = "AV Remote")]
    AvRemote,
    #[debug(format = "AV Device")]
    AvDevice,
}

impl Parsable for ZWaveLibraryType {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "ZWaveLibraryType",
            map(be_u8, |x| ZWaveLibraryType::try_from(x).unwrap()),
        )(i)
    }
}

impl Serializable for ZWaveLibraryType {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
