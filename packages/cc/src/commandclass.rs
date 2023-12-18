use enum_dispatch::enum_dispatch;
use typed_builder::TypedBuilder;
use zwave_core::{encoding::Input, prelude::*, submodule};

use crate::commandclass_raw::CCRaw;

submodule!(basic);

#[derive(Default, Clone, PartialEq, TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CCParsingContext {
    #[builder(setter(strip_option))]
    origin: Option<MessageOrigin>,
    #[builder(setter(strip_option))]
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

// FIXME: auto-implement From<CCImpl> for CC

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
/// Command-specific functionality that may need to be implemented for each command
pub trait CCBase: std::fmt::Debug + Sync + Send {}

pub trait CCRequest: CCId {
    fn expects_response(&self) -> bool;
    fn test_response(&self, _response: &CC) -> bool {
        // FIXME:
        todo!("Implement default test_response for {:?}", self)
    }
}

pub struct CCAddress {
    /// The source node of this CC
    pub source_node: NodeId,
    /// The destination node(s) of this CC
    pub target_node: Destination,
    /// Which endpoint of the node this CC belongs to
    pub endpoint_index: EndpointIndex,
}

pub struct CCInfo {
    /// The version of the specification this CC was parsed with
    pub version: u8,
}

/// Defines the destination of a command class
pub enum Destination {
    Singlecast(NodeId),
    Multicast(Vec<NodeId>),
    Broadcast,
}

pub enum EndpointIndex {
    Root,
    Endpoint(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotImplemented {
    pub cc_id: CommandClasses,
    pub cc_command: Option<u8>,
    // #[debug(with = "hex_fmt")]
    pub payload: Vec<u8>,
}

impl CCBase for NotImplemented {}

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
