use crate::{frame::SerialFrame, prelude::*};
use enum_dispatch::enum_dispatch;

mod capability;
pub use capability::*;

#[enum_dispatch(Command)]
pub trait CommandBase {
    fn command_type(&self) -> CommandType;
    fn function_type(&self) -> FunctionType;
}

define_commands!(
    GetSerialApiInitDataRequest {
        command_type: CommandType::Request,
        function_type: FunctionType::GetSerialApiInitData,
    },
    GetSerialApiInitDataResponse {
        command_type: CommandType::Response,
        function_type: FunctionType::GetSerialApiInitData,
    }
);

pub trait CommandRequest {
    fn expects_response(&self) -> bool;
    fn test_response(&self, response: &Command) -> bool;
    fn expects_callback(&self) -> bool;
    fn test_callback(&self, callback: &Command) -> bool;

    fn callback_id(&self) -> Option<u8>;
    fn set_callback_id(&mut self, callback_id: Option<u8>);
    fn needs_callback_id(&self) -> bool {
        true
    }
}

macro_rules! define_commands {
    (
        $( $cmd_name:ident {
            command_type: CommandType::$cmd_type:ident,
            function_type: FunctionType::$fn_type:ident,
        } ),+
    ) => {
        // Define the command enum with all possible variants.
        // Calls to the command enum will be dispatched to the corresponding variant.
        #[enum_dispatch]
        pub enum Command {
            $( $cmd_name($cmd_name) ),+
        }

        // Define command type and function type for each variant
        $( impl CommandBase for $cmd_name {
            fn command_type(&self) -> CommandType {
                CommandType::$cmd_type
            }

            fn function_type(&self) -> FunctionType {
                FunctionType::$fn_type
            }
        } )+

        // Delegate Serialization to the corresponding variant
        impl Serializable for Command {
            fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
                move |out| match self {
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
                type Error = crate::error::Error;

                fn try_into(self) -> std::result::Result<CommandRaw, Self::Error> {
                    let cmd: Command = self.into();
                    cmd.try_into()
                }
            }

            impl TryInto<SerialFrame> for $cmd_name {
                type Error = crate::error::Error;

                fn try_into(self) -> std::result::Result<SerialFrame, Self::Error> {
                    let raw: CommandRaw = self.try_into()?;
                    Ok(raw.into())
                }
            }
        )+

        // Implement conversion from a raw command to the correct variant
        impl TryFrom<CommandRaw> for Command {
            type Error = crate::error::Error;

            fn try_from(raw: CommandRaw) -> std::result::Result<Self, Self::Error> {
                let command_type = raw.command_type;
                let function_type = raw.function_type;
                let raw_payload = raw.payload.as_slice();

                match (command_type, function_type) {
                    $( (CommandType::$cmd_type, FunctionType::$fn_type) => {
                        Ok(Command::$cmd_name($cmd_name::try_from(raw_payload)?))
                    } )+
                    _ => todo!("Implement Command variant for NotImplemented"),
                }
            }
        }

    };
}

pub(crate) use define_commands;
