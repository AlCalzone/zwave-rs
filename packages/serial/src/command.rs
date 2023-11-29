use crate::prelude::*;
use zwave_core::prelude::*;

use crate::{frame::SerialFrame, util::hex_fmt};
use custom_debug_derive::Debug;
use enum_dispatch::enum_dispatch;
use zwave_core::{impl_vec_conversion_for, impl_vec_parsing_for, impl_vec_serializing_for};

mod capability;
pub use capability::*;
mod misc;
pub use misc::*;
mod transport;
pub use transport::*;

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

define_commands!(
    GetSerialApiInitDataRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::GetSerialApiInitData,
        origin: MessageOrigin::Host,
    },
    GetSerialApiInitDataResponse {
        command_type: CommandType::Response,
        function_type: FunctionType::GetSerialApiInitData,
        origin: MessageOrigin::Controller,
    },
    SoftResetRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::SoftReset,
        origin: MessageOrigin::Host,
    },
    GetControllerVersionRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::GetControllerVersion,
        origin: MessageOrigin::Host,
    },
    GetControllerVersionResponse {
        command_type: CommandType::Response,
        function_type: FunctionType::GetControllerVersion,
        origin: MessageOrigin::Controller,
    },
    GetProtocolVersionRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::GetProtocolVersion,
        origin: MessageOrigin::Host,
    },
    GetProtocolVersionResponse {
        command_type: CommandType::Response,
        function_type: FunctionType::GetProtocolVersion,
        origin: MessageOrigin::Controller,
    },
    SendDataRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::SendData,
        origin: MessageOrigin::Host,
    },
    SendDataResponse {
        command_type: CommandType::Response,
        function_type: FunctionType::SendData,
        origin: MessageOrigin::Controller,
    },
    SendDataCallback {
        command_type: CommandType::Request,
        function_type: FunctionType::SendData,
        origin: MessageOrigin::Controller,
    },
    BridgeApplicationCommandRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::BridgeApplicationCommand,
        origin: MessageOrigin::Controller,
    },
);

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

macro_rules! define_commands {
    (
        $( $cmd_name:ident {
            command_type: CommandType::$cmd_type:ident,
            function_type: FunctionType::$fn_type:ident,
            origin: MessageOrigin::$origin:ident,
        } ),+ $(,)? // trailing comma
    ) => {
        // Define the command enum with all possible variants.
        // Calls to the command enum will be dispatched to the corresponding variant.
        #[enum_dispatch]
        #[derive(Debug, Clone, PartialEq)]
        pub enum Command {
            NotImplemented(NotImplemented),
            $( $cmd_name($cmd_name) ),+
        }

        // Define command type and function type for each variant
        $( impl CommandId for $cmd_name {
            fn command_type(&self) -> CommandType {
                CommandType::$cmd_type
            }

            fn function_type(&self) -> FunctionType {
                FunctionType::$fn_type
            }

            fn origin(&self) -> MessageOrigin {
                MessageOrigin::$origin
            }
        } )+

        // Delegate Serialization to the corresponding variant
        impl Serializable for Command {
            fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
                move |out| match self {
                    Self::NotImplemented(c) => cookie_factory::combinator::slice(&c.payload)(out),
                    $( Self::$cmd_name(c) => c.serialize()(out), )+
                }
            }
        }


        // Implement the default TryFrom<&[u8]>/TryInto<Vec<u8>> conversions for each variant
        $(
            impl_vec_conversion_for!($cmd_name);
        )+

        // Implement shortcuts from each variant to CommandRaw / SerialFrame
        $(
            impl TryInto<CommandRaw> for $cmd_name {
                type Error = EncodingError;

                fn try_into(self) -> std::result::Result<CommandRaw, Self::Error> {
                    let cmd: Command = self.into();
                    cmd.try_into()
                }
            }

            impl From<$cmd_name> for SerialFrame {
                fn from(val: $cmd_name) -> Self {
                    SerialFrame::Command(val.into())
                }
            }
        )+

        // Implement conversion from a raw command to the correct variant
        impl TryFrom<CommandRaw> for Command {
            type Error = EncodingError;

            fn try_from(raw: CommandRaw) -> std::result::Result<Self, Self::Error> {
                let command_type = raw.command_type;
                let function_type = raw.function_type;
                // We parse commands that are sent by the controller
                let expected_origin = MessageOrigin::Controller;

                // ...and hope that Rust optimizes the match arms with origin Host away
                match (command_type, function_type, expected_origin) {
                    $( (CommandType::$cmd_type, FunctionType::$fn_type, MessageOrigin::$origin) => {
                        Ok(Self::$cmd_name($cmd_name::try_from(raw.payload.as_slice())?))
                    } )+
                    _ => Ok(Self::NotImplemented(NotImplemented {
                        command_type,
                        function_type,
                        payload: raw.payload,
                    })),
                }
            }
        }

    };
}
use define_commands;

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
