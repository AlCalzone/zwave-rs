use std::{
    marker::Sized,
    ops::{Deref, DerefMut},
};

use enum_dispatch::enum_dispatch;
use typed_builder::TypedBuilder;
use zwave_core::{cache::CacheValue, encoding::Input, prelude::*, value_id::ValueId};

use crate::commandclass_raw::CCRaw;

#[derive(Default, Clone, PartialEq, TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CCParsingContext {
    #[builder(default, setter(into))]
    frame_addressing: Option<FrameAddressing>,
}

pub trait CCParsable
where
    Self: Sized + CCBase,
{
    fn parse<'a>(i: Input<'a>, ctx: &CCParsingContext) -> ParseResult<'a, Self>;

    fn try_from_slice(data: &[u8], ctx: &CCParsingContext) -> Result<Self, EncodingError> {
        Self::parse(data, ctx).into_encoding_result()
    }
}

pub trait CCSerializable
where
    Self: Sized,
{
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a;

    fn try_to_vec(&self) -> Result<Vec<u8>, EncodingError> {
        cookie_factory::gen_simple(self.serialize(), Vec::new()).into_encoding_result()
    }
}

// This auto-generates the CC enum by reading the files in the given directory
// and extracting the information from the CCId impls.
proc_macros::impl_cc_enum!("src/commandclass");

#[enum_dispatch(CC)]
/// Identifies a command class and its commands
pub trait CCId: CCBase {
    /// The command class identifier
    fn cc_id(&self) -> CommandClasses;

    /// The subcommand identifier, if applicable
    /// FIXME: Figure out an ergonomic way to work with CC specific command enums
    fn cc_command(&self) -> Option<u8>;

    // Which version of the CC is implemented by this library
    // FIXME: This does not belong on the individual commands
    // fn implemented_version(&self) -> u8;
}

#[enum_dispatch(CC)]
pub trait CCValues {
    fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
        // CCs which carry values should implement this. For all others, this is a no-op.
        vec![]
    }
}

#[enum_dispatch(CC)]
/// Command-specific functionality that may need to be implemented for each command
pub trait CCBase: CCValues + std::fmt::Debug + Sync + Send {}

pub trait CCRequest: CCId + Sized {
    fn expects_response(&self) -> bool;
    fn test_response(&self, response: &CC) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithAddress<T: CCBase> {
    address: CCAddress,
    command: T,
}

impl<T> WithAddress<T>
where
    T: CCBase,
{
    pub fn address(&self) -> &CCAddress {
        &self.address
    }

    pub fn set_address(&mut self, address: CCAddress) {
        self.address = address;
    }

    pub fn with_destination(self, destination: Destination) -> Self {
        let mut address = self.address;
        address.destination = destination;

        Self { address, ..self }
    }

    pub fn with_endpoint_index(self, endpoint_index: EndpointIndex) -> Self {
        let mut address = self.address;
        address.endpoint_index = endpoint_index;

        Self { address, ..self }
    }

    pub fn with_source_node_id(self, source_node_id: NodeId) -> Self {
        let mut address = self.address;
        address.source_node_id = source_node_id;

        Self { address, ..self }
    }

    pub fn unwrap(self) -> T {
        self.command
    }
}

impl<T> Deref for WithAddress<T>
where
    T: CCBase,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl<T> DerefMut for WithAddress<T>
where
    T: CCBase,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}

impl<F> From<WithAddress<F>> for CC
where
    CC: From<F>,
    F: CCBase,
{
    fn from(val: WithAddress<F>) -> Self {
        Self::from(val.command)
    }
}

pub trait CCAddressable {
    fn with_address(self, address: CCAddress) -> WithAddress<Self>
    where
        Self: Sized + CCBase,
    {
        WithAddress {
            address,
            command: self,
        }
    }

    fn with_destination(self, destination: Destination) -> WithAddress<Self>
    where
        Self: Sized + CCBase,
    {
        self.with_address(CCAddress {
            destination,
            ..Default::default()
        })
    }

    fn clone_with_address(&self, address: CCAddress) -> WithAddress<Self>
    where
        Self: Sized + CCBase + Clone,
    {
        WithAddress {
            address,
            command: self.clone(),
        }
    }

    fn clone_with_destination(&self, destination: Destination) -> WithAddress<Self>
    where
        Self: Sized + CCBase + Clone,
    {
        self.clone_with_address(CCAddress {
            destination,
            ..Default::default()
        })
    }
}

impl<T> CCAddressable for T where T: CCBase {}

#[derive(Debug, Clone, PartialEq)]
pub struct CCAddress {
    /// The source node of this CC
    pub source_node_id: NodeId,
    /// The destination node(s) of this CC
    pub destination: Destination,
    /// Which endpoint of the node this CC belongs to
    pub endpoint_index: EndpointIndex,
}

impl Default for CCAddress {
    fn default() -> Self {
        // The default for the CC address is not terribly useful,
        // but it makes working with it less cumbersome
        Self {
            source_node_id: NodeId::unspecified(),
            destination: Destination::Singlecast(NodeId::unspecified()),
            endpoint_index: EndpointIndex::Root,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CCInfo {
    /// The version of the specification this CC was parsed with
    pub version: u8,
}

/// Defines the destination of a command class
#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Singlecast(NodeId),
    Multicast(Vec<NodeId>),
    Broadcast,
}

macro_rules! impl_destination_conversions_for {
    ($t:ty) => {
        impl From<$t> for Destination {
            fn from(val: $t) -> Self {
                Self::Singlecast(val.into())
            }
        }

        impl PartialEq<$t> for Destination {
            fn eq(&self, other: &$t) -> bool {
                self == &Destination::from(*other)
            }
        }
    };
}

impl_destination_conversions_for!(u8);
impl_destination_conversions_for!(u16);
impl_destination_conversions_for!(i32);
impl_destination_conversions_for!(NodeId);

#[derive(Debug, Clone, PartialEq)]
pub struct NotImplemented {
    pub cc_id: CommandClasses,
    pub cc_command: Option<u8>,
    // #[debug(with = "hex_fmt")]
    pub payload: Vec<u8>,
}

impl CCBase for NotImplemented {}

impl CCValues for NotImplemented {}

impl CCId for NotImplemented {
    fn cc_id(&self) -> CommandClasses {
        self.cc_id
    }

    fn cc_command(&self) -> Option<u8> {
        self.cc_command
    }
}

#[test]
fn test_cc_try_from_raw() {
    let raw = CCRaw {
        cc_id: CommandClasses::Basic,
        cc_command: Some(BasicCCCommand::Get as _),
        payload: vec![],
    };

    let ctx = CCParsingContext::default();
    let cc = CC::try_from_raw(raw, &ctx).unwrap();
    assert_eq!(cc, CC::BasicCCGet(BasicCCGet::default()));
}

#[test]
fn test_cc_try_into_raw() {
    let cc = CC::NotImplemented(NotImplemented {
        cc_id: CommandClasses::Basic,
        cc_command: Some(0x01u8),
        payload: vec![0x02u8, 0x03],
    });
    let raw: CCRaw = cc.try_into_raw().unwrap();

    assert_eq!(
        raw,
        CCRaw {
            cc_id: CommandClasses::Basic,
            cc_command: Some(0x01u8),
            payload: vec![0x02u8, 0x03]
        }
    );
}
