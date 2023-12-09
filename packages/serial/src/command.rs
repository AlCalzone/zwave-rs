use crate::prelude::*;
use zwave_core::{encoding::Input, prelude::*, submodule};

use crate::{frame::SerialFrame, util::hex_fmt};
use custom_debug_derive::Debug;
use enum_dispatch::enum_dispatch;
use zwave_core::{impl_vec_parsing_with_context_for, impl_vec_serializing_for};

submodule!(application);
submodule!(capability);
submodule!(misc);
submodule!(transport);

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandParseContext {
    sdk_version: Option<Version>,
}

pub trait CommandParsable
where
    Self: Sized + CommandBase,
{
    fn parse(i: Input, ctx: CommandParseContext) -> ParseResult<Self>;
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
