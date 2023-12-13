use crate::prelude::*;
use derive_builder::Builder;
use zwave_core::{encoding::Input, prelude::*, submodule};

use crate::{frame::SerialFrame, util::hex_fmt};
use custom_debug_derive::Debug;
use enum_dispatch::enum_dispatch;

submodule!(application);
submodule!(capability);
submodule!(misc);
submodule!(transport);
submodule!(network_mgmt);

#[derive(Default, Debug, Clone, PartialEq, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct CommandEncodingContext {
    sdk_version: Option<Version>,
    node_id_type: NodeIdType,
}

impl CommandEncodingContext {
    pub fn builder() -> CommandEncodingContextBuilder {
        CommandEncodingContextBuilder::default()
    }
}

pub trait CommandParsable
where
    Self: Sized + CommandBase,
{
    fn parse<'a>(i: Input<'a>, ctx: &CommandEncodingContext) -> ParseResult<'a, Self>;

    fn try_from_slice(data: &[u8], ctx: &CommandEncodingContext) -> Result<Self, EncodingError> {
        Self::parse(data, ctx).into_encoding_result()
    }
}

pub trait CommandSerializable
where
    Self: Sized,
{
    fn serialize<'a, W: std::io::Write + 'a>(
        &'a self,
        ctx: &'a CommandEncodingContext,
    ) -> impl cookie_factory::SerializeFn<W> + 'a;

    fn try_to_vec<'a>(&'a self, ctx: &'a CommandEncodingContext) -> Result<Vec<u8>, EncodingError> {
        cookie_factory::gen_simple(self.serialize(ctx), Vec::new()).into_encoding_result()
    }
}

#[enum_dispatch(Command)]
/// Command-specific functionality that may need to be implemented for each command
pub trait CommandBase: std::fmt::Debug + Sync + Send {
    // Used to test responses and callbacks whether they indicate an OK result
    fn is_ok(&self) -> bool {
        true
    }

    // Commands may or may not have a callback ID
    fn callback_id(&self) -> Option<u8> {
        None
    }
}

#[enum_dispatch(Command)]
/// Identifies the types of a command
pub trait CommandId: CommandBase {
    fn command_type(&self) -> CommandType;
    fn function_type(&self) -> FunctionType;
    fn origin(&self) -> MessageOrigin;
}

// This auto-generates the Command enum by reading the files in the given directory
// and extracting the information from the CommandId impls.
proc_macros::impl_command_enum!("src/command");

pub trait CommandRequest: CommandId {
    fn expects_response(&self) -> bool;
    fn test_response(&self, response: &Command) -> bool {
        // By default, we expect a response with the same function type
        self.expects_response()
            && response.command_type() == CommandType::Response
            && response.function_type() == self.function_type()
    }

    fn expects_callback(&self) -> bool;
    fn test_callback(&self, callback: &Command) -> bool {
        // By default, we expect a callback with the same function type
        if self.expects_callback()
            && callback.command_type() == CommandType::Request
            && callback.function_type() == self.function_type()
        {
            // We may have to check the callback ID
            if self.needs_callback_id() {
                let callback_id = self.callback_id().unwrap_or_else(|| {
                    panic!("Command {:?} needs a callback ID, but none was set", self)
                });
                callback.callback_id() == Some(callback_id)
            } else {
                true
            }
        } else {
            false
        }
    }

    // By default: don't need a callback
    fn needs_callback_id(&self) -> bool {
        false
    }
    fn set_callback_id(&mut self, _callback_id: Option<u8>) {}
}

impl From<Command> for SerialFrame {
    fn from(val: Command) -> Self {
        SerialFrame::Command(val)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotImplemented {
    pub command_type: CommandType,
    pub function_type: FunctionType,
    #[debug(with = "hex_fmt")]
    pub payload: Vec<u8>,
}

impl CommandBase for NotImplemented {}

impl CommandId for NotImplemented {
    fn command_type(&self) -> CommandType {
        self.command_type
    }

    fn function_type(&self) -> FunctionType {
        self.function_type
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}
