use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::TryFromRepr;
use zwave_core::bake::{self, Encoder, EncoderWith};
use zwave_core::encoding::{parser_not_implemented, parsers::fixed_length_bitmask_u8};
use zwave_core::munch::{
    bytes::{be_i16, be_i8, be_u8, complete::skip},
    combinators::{map, map_res},
};
use zwave_core::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum SerialApiSetupCommand {
    Unsupported = 0x00,
    GetSupportedCommands = 0x01,
    SetTxStatusReport = 0x02,
    SetPowerlevel = 0x04,
    GetPowerlevel = 0x08,
    GetMaximumPayloadSize = 0x10,
    GetRFRegion = 0x20,
    SetRFRegion = 0x40,
    SetNodeIDType = 0x80,

    // These are added "inbetween" the existing commands
    SetLRMaximumTxPower = 0x03,
    GetLRMaximumTxPower = 0x05,
    GetLRMaximumPayloadSize = 0x11,
    SetPowerlevel16Bit = 0x12,
    GetPowerlevel16Bit = 0x13,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SerialApiSetupRequest {
    command: SerialApiSetupCommand,
    payload: SerialApiSetupRequestPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SerialApiSetupRequestPayload {
    GetSupportedCommands,
    SetTxStatusReport { enabled: bool },
    SetPowerlevel { powerlevel: Powerlevel },
    GetPowerlevel,
    GetMaximumPayloadSize,
    GetRFRegion,
    SetRFRegion { region: RfRegion },
    SetNodeIDType { node_id_type: NodeIdType },
    SetLRMaximumTxPower { max_power: f32 },
    GetLRMaximumTxPower,
    GetLRMaximumPayloadSize,
    SetPowerlevel16Bit { powerlevel: Powerlevel },
    GetPowerlevel16Bit,
}

impl SerialApiSetupRequest {
    pub fn get_supported_commands() -> Self {
        Self {
            command: SerialApiSetupCommand::GetSupportedCommands,
            payload: SerialApiSetupRequestPayload::GetSupportedCommands,
        }
    }

    pub fn set_tx_status_report(enabled: bool) -> Self {
        Self {
            command: SerialApiSetupCommand::SetTxStatusReport,
            payload: SerialApiSetupRequestPayload::SetTxStatusReport { enabled },
        }
    }

    pub fn set_powerlevel(powerlevel: Powerlevel) -> Self {
        Self {
            command: SerialApiSetupCommand::SetPowerlevel,
            payload: SerialApiSetupRequestPayload::SetPowerlevel { powerlevel },
        }
    }

    pub fn get_powerlevel() -> Self {
        Self {
            command: SerialApiSetupCommand::GetPowerlevel,
            payload: SerialApiSetupRequestPayload::GetPowerlevel,
        }
    }

    pub fn get_maximum_payload_size() -> Self {
        Self {
            command: SerialApiSetupCommand::GetMaximumPayloadSize,
            payload: SerialApiSetupRequestPayload::GetMaximumPayloadSize,
        }
    }

    pub fn get_rf_region() -> Self {
        Self {
            command: SerialApiSetupCommand::GetRFRegion,
            payload: SerialApiSetupRequestPayload::GetRFRegion,
        }
    }

    pub fn set_rf_region(region: RfRegion) -> Self {
        Self {
            command: SerialApiSetupCommand::SetRFRegion,
            payload: SerialApiSetupRequestPayload::SetRFRegion { region },
        }
    }

    pub fn set_node_id_type(node_id_type: NodeIdType) -> Self {
        Self {
            command: SerialApiSetupCommand::SetNodeIDType,
            payload: SerialApiSetupRequestPayload::SetNodeIDType { node_id_type },
        }
    }

    pub fn set_lr_maximum_tx_power(max_power: f32) -> Self {
        Self {
            command: SerialApiSetupCommand::SetLRMaximumTxPower,
            payload: SerialApiSetupRequestPayload::SetLRMaximumTxPower { max_power },
        }
    }

    pub fn get_lr_maximum_tx_power() -> Self {
        Self {
            command: SerialApiSetupCommand::GetLRMaximumTxPower,
            payload: SerialApiSetupRequestPayload::GetLRMaximumTxPower,
        }
    }

    pub fn get_lr_maximum_payload_size() -> Self {
        Self {
            command: SerialApiSetupCommand::GetLRMaximumPayloadSize,
            payload: SerialApiSetupRequestPayload::GetLRMaximumPayloadSize,
        }
    }

    pub fn set_powerlevel_16bit(powerlevel: Powerlevel) -> Self {
        Self {
            command: SerialApiSetupCommand::SetPowerlevel16Bit,
            payload: SerialApiSetupRequestPayload::SetPowerlevel16Bit { powerlevel },
        }
    }

    pub fn get_powerlevel_16bit() -> Self {
        Self {
            command: SerialApiSetupCommand::GetPowerlevel16Bit,
            payload: SerialApiSetupRequestPayload::GetPowerlevel16Bit,
        }
    }
}

impl CommandId for SerialApiSetupRequest {
    fn command_type(&self) -> CommandType {
        CommandType::Request
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SerialApiSetup
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Host
    }
}

impl CommandBase for SerialApiSetupRequest {}

impl CommandRequest for SerialApiSetupRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &Command) -> bool {
        if let Command::SerialApiSetupResponse(res) = response {
            return self.command == res.command;
        }
        false
    }

    fn expects_callback(&self) -> bool {
        false
    }
}

impl CommandParsable for SerialApiSetupRequest {
    fn parse(_i: &mut Bytes, _ctx: &CommandEncodingContext) -> MunchResult<Self> {
        parser_not_implemented("ERROR: SerialApiSetupRequest::parse() not implemented")
        // Ok(Self {})
    }
}

impl EncoderWith<&CommandEncodingContext> for SerialApiSetupRequest {
    fn write(&self, output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        use bake::{
            bytes::{be_i16, be_i8, be_u8},
            sequence::tuple,
        };

        be_u8(self.command as u8).write(output);
        match self.payload {
            SerialApiSetupRequestPayload::GetSupportedCommands
            | SerialApiSetupRequestPayload::GetPowerlevel
            | SerialApiSetupRequestPayload::GetMaximumPayloadSize
            | SerialApiSetupRequestPayload::GetRFRegion
            | SerialApiSetupRequestPayload::GetLRMaximumTxPower
            | SerialApiSetupRequestPayload::GetLRMaximumPayloadSize
            | SerialApiSetupRequestPayload::GetPowerlevel16Bit => {
                // No payload
            }

            SerialApiSetupRequestPayload::SetTxStatusReport { enabled } => {
                be_u8(if enabled { 0xff } else { 0x00 }).write(output)
            }
            SerialApiSetupRequestPayload::SetPowerlevel {
                powerlevel:
                    Powerlevel {
                        tx_power: tx_power_dbm,
                        measured_at_0_dbm,
                    },
            } => tuple((
                // The values are represented as a multiple of 0.1 dBm
                be_i8((tx_power_dbm * 10f32).round() as i8),
                be_i8((measured_at_0_dbm * 10f32).round() as i8),
            ))
            .write(output),
            SerialApiSetupRequestPayload::SetPowerlevel16Bit {
                powerlevel:
                    Powerlevel {
                        tx_power: tx_power_dbm,
                        measured_at_0_dbm,
                    },
            } => tuple((
                // The values are represented as a multiple of 0.1 dBm
                be_i16((tx_power_dbm * 10f32).round() as i16),
                be_i16((measured_at_0_dbm * 10f32).round() as i16),
            ))
            .write(output),
            SerialApiSetupRequestPayload::SetLRMaximumTxPower { max_power } => {
                // The values are represented as a multiple of 0.1 dBm
                be_i16((max_power * 10f32).round() as i16).write(output)
            }
            SerialApiSetupRequestPayload::SetRFRegion { region } => region.write(output),
            SerialApiSetupRequestPayload::SetNodeIDType { node_id_type } => {
                node_id_type.write(output)
            }
        }
    }
}

impl ToLogPayload for SerialApiSetupRequest {
    fn to_log_payload(&self) -> LogPayload {
        let ret = LogPayloadDict::new().with_entry("command", format!("{:?}", self.command));

        let additional = match self.payload {
            SerialApiSetupRequestPayload::SetTxStatusReport { enabled } => {
                LogPayloadDict::new().with_entry("enabled", enabled)
            }
            SerialApiSetupRequestPayload::SetPowerlevel { powerlevel } => {
                LogPayloadDict::new().with_entry("powerlevel", powerlevel.to_string())
            }
            SerialApiSetupRequestPayload::SetRFRegion { region } => {
                LogPayloadDict::new().with_entry("region", region.to_string())
            }
            SerialApiSetupRequestPayload::SetNodeIDType { node_id_type } => {
                LogPayloadDict::new().with_entry("node ID type", node_id_type.to_string())
            }
            SerialApiSetupRequestPayload::SetLRMaximumTxPower { max_power } => {
                LogPayloadDict::new().with_entry("max. TX power", format!("{:.1} dBm", max_power))
            }
            SerialApiSetupRequestPayload::SetPowerlevel16Bit { powerlevel } => {
                LogPayloadDict::new().with_entry("powerlevel", powerlevel.to_string())
            }
            _ => LogPayloadDict::new(),
        };

        ret.extend(additional).into()
    }
}

// #[test]
// fn test_round() {
//     let val: f32 = 12.61f32;
//     let i = (val * 10f32).round() as i8;
//     println!("{}", i);
// }

#[derive(Debug, Clone, PartialEq)]
pub struct SerialApiSetupResponse {
    pub command: SerialApiSetupCommand,
    pub payload: SerialApiSetupResponsePayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SerialApiSetupResponsePayload {
    Unsupported(SerialApiSetupCommand),
    GetSupportedCommands {
        commands: Vec<SerialApiSetupCommand>,
    },
    SetTxStatusReport {
        success: bool,
    },
    SetPowerlevel {
        success: bool,
    },
    GetPowerlevel {
        powerlevel: Powerlevel,
    },
    GetMaximumPayloadSize {
        size: u8,
    },
    GetRFRegion {
        region: RfRegion,
    },
    SetRFRegion {
        success: bool,
    },
    SetNodeIDType {
        success: bool,
    },
    SetLRMaximumTxPower {
        success: bool,
    },
    GetLRMaximumTxPower {
        max_power: f32,
    },
    GetLRMaximumPayloadSize {
        size: u8,
    },
    SetPowerlevel16Bit {
        success: bool,
    },
    GetPowerlevel16Bit {
        powerlevel: Powerlevel,
    },
}

impl CommandId for SerialApiSetupResponse {
    fn command_type(&self) -> CommandType {
        CommandType::Response
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::SerialApiSetup
    }

    fn origin(&self) -> MessageOrigin {
        MessageOrigin::Controller
    }
}

impl CommandBase for SerialApiSetupResponse {}

impl CommandParsable for SerialApiSetupResponse {
    fn parse(i: &mut Bytes, ctx: &CommandEncodingContext) -> MunchResult<Self> {
        let command = map_res(be_u8, SerialApiSetupCommand::try_from).parse(i)?;
        let payload = match command {
            SerialApiSetupCommand::Unsupported => {
                SerialApiSetupResponsePayload::Unsupported(command)
            }
            SerialApiSetupCommand::GetSupportedCommands => {
                let mut commands = if i.len() > 1 {
                    // This module supports the extended bitmask to report the supported serial API setup commands
                    // Ignore the first byte and parse the rest as a bitmask
                    skip(1usize).parse(i)?;

                    // According to the Host API specification, the first bit (bit 0) should be GetSupportedCommands
                    // However, in Z-Wave SDK < 7.19.1, the entire bitmask is shifted by 1 bit and
                    // GetSupportedCommands is encoded in the second bit (bit 1)
                    let start_value =
                        if ctx.sdk_version < Some(Version::try_from("7.19.1").unwrap()) {
                            SerialApiSetupCommand::Unsupported
                        } else {
                            SerialApiSetupCommand::GetSupportedCommands
                        };

                    map_res(
                        move |i: &mut Bytes| fixed_length_bitmask_u8(i, start_value as u8, i.len()),
                        |x| {
                            x.iter()
                                .map(|x| SerialApiSetupCommand::try_from(*x))
                                .collect::<Result<Vec<_>, _>>()
                        },
                    )
                    .parse(i)?
                } else {
                    // This module only uses the single byte power-of-2 bitmask. Decode it manually.
                    let bitmask = be_u8(i)?;
                    let commands = [
                        SerialApiSetupCommand::GetSupportedCommands,
                        SerialApiSetupCommand::SetTxStatusReport,
                        SerialApiSetupCommand::SetPowerlevel,
                        SerialApiSetupCommand::GetPowerlevel,
                        SerialApiSetupCommand::GetMaximumPayloadSize,
                        SerialApiSetupCommand::GetRFRegion,
                        SerialApiSetupCommand::SetRFRegion,
                        SerialApiSetupCommand::SetNodeIDType,
                    ];

                    commands
                        .into_iter()
                        .filter(|x| {
                            let x = *x as u8;
                            (bitmask & x) == x
                        })
                        .collect()
                };

                // Apparently GetSupportedCommands is not always included in the bitmask, although we
                // just received a response to the command
                if !commands.contains(&SerialApiSetupCommand::GetSupportedCommands) {
                    commands.insert(0, SerialApiSetupCommand::GetSupportedCommands);
                }

                SerialApiSetupResponsePayload::GetSupportedCommands { commands }
            }

            SerialApiSetupCommand::SetTxStatusReport => {
                let success = map(be_u8, |x| x > 0).parse(i)?;
                SerialApiSetupResponsePayload::SetTxStatusReport { success }
            }

            SerialApiSetupCommand::SetPowerlevel => {
                let success = map(be_u8, |x| x > 0).parse(i)?;
                SerialApiSetupResponsePayload::SetPowerlevel { success }
            }
            SerialApiSetupCommand::GetPowerlevel => {
                let tx_power_dbm = map(be_i8, |x| x as f32 / 10f32).parse(i)?;
                let measured_at_0_dbm = map(be_i8, |x| x as f32 / 10f32).parse(i)?;
                SerialApiSetupResponsePayload::GetPowerlevel {
                    powerlevel: Powerlevel {
                        tx_power: tx_power_dbm,
                        measured_at_0_dbm,
                    },
                }
            }

            SerialApiSetupCommand::GetMaximumPayloadSize => {
                let size = be_u8(i)?;
                SerialApiSetupResponsePayload::GetMaximumPayloadSize { size }
            }
            SerialApiSetupCommand::GetRFRegion => {
                let region = RfRegion::parse(i)?;
                SerialApiSetupResponsePayload::GetRFRegion { region }
            }
            SerialApiSetupCommand::SetRFRegion => {
                let success = map(be_u8, |x| x > 0).parse(i)?;
                SerialApiSetupResponsePayload::SetRFRegion { success }
            }
            SerialApiSetupCommand::SetNodeIDType => {
                let success = map(be_u8, |x| x > 0).parse(i)?;
                SerialApiSetupResponsePayload::SetNodeIDType { success }
            }
            SerialApiSetupCommand::SetLRMaximumTxPower => {
                let success = map(be_u8, |x| x > 0).parse(i)?;
                SerialApiSetupResponsePayload::SetLRMaximumTxPower { success }
            }
            SerialApiSetupCommand::GetLRMaximumTxPower => {
                let max_power = map(be_i16, |x| x as f32 / 10f32).parse(i)?;
                SerialApiSetupResponsePayload::GetLRMaximumTxPower { max_power }
            }
            SerialApiSetupCommand::GetLRMaximumPayloadSize => {
                let size = be_u8(i)?;
                SerialApiSetupResponsePayload::GetLRMaximumPayloadSize { size }
            }
            SerialApiSetupCommand::SetPowerlevel16Bit => {
                let success = map(be_u8, |x| x > 0).parse(i)?;
                SerialApiSetupResponsePayload::SetPowerlevel16Bit { success }
            }
            SerialApiSetupCommand::GetPowerlevel16Bit => {
                let tx_power_dbm = map(be_i16, |x| x as f32 / 10f32).parse(i)?;
                let measured_at_0_dbm = map(be_i16, |x| x as f32 / 10f32).parse(i)?;

                SerialApiSetupResponsePayload::GetPowerlevel16Bit {
                    powerlevel: Powerlevel {
                        tx_power: tx_power_dbm,
                        measured_at_0_dbm,
                    },
                }
            }
        };
        Ok(Self { command, payload })
    }
}

impl EncoderWith<&CommandEncodingContext> for SerialApiSetupResponse {
    fn write(&self, _output: &mut BytesMut, _ctx: &CommandEncodingContext) {
        todo!("ERROR: SerialApiSetupResponse::write() not implemented");
    }
}

impl ToLogPayload for SerialApiSetupResponse {
    fn to_log_payload(&self) -> LogPayload {
        let ret = LogPayloadDict::new().with_entry("command", format!("{:?}", self.command));

        let additional = match self.payload {
            SerialApiSetupResponsePayload::Unsupported(ref c) => {
                return LogPayloadText::new(format!("Unsupported command: {:?}", c)).into()
            }
            SerialApiSetupResponsePayload::GetSupportedCommands { ref commands } => {
                LogPayloadDict::new().with_entry(
                    "supported commands",
                    LogPayloadList::new(commands.iter().map(|cmd| format!("{:?}", cmd).into())),
                )
            }
            SerialApiSetupResponsePayload::SetTxStatusReport { success }
            | SerialApiSetupResponsePayload::SetPowerlevel { success }
            | SerialApiSetupResponsePayload::SetRFRegion { success }
            | SerialApiSetupResponsePayload::SetNodeIDType { success }
            | SerialApiSetupResponsePayload::SetLRMaximumTxPower { success }
            | SerialApiSetupResponsePayload::SetPowerlevel16Bit { success } => {
                LogPayloadDict::new().with_entry("success", success)
            }
            SerialApiSetupResponsePayload::GetPowerlevel { powerlevel }
            | SerialApiSetupResponsePayload::GetPowerlevel16Bit { powerlevel } => {
                LogPayloadDict::new().with_entry("powerlevel", powerlevel.to_string())
            }
            SerialApiSetupResponsePayload::GetMaximumPayloadSize { size }
            | SerialApiSetupResponsePayload::GetLRMaximumPayloadSize { size } => {
                LogPayloadDict::new().with_entry("max. payload size", size)
            }
            SerialApiSetupResponsePayload::GetRFRegion { region } => {
                LogPayloadDict::new().with_entry("region", region.to_string())
            }
            SerialApiSetupResponsePayload::GetLRMaximumTxPower { max_power } => {
                LogPayloadDict::new().with_entry("max. TX power", format!("{:.1} dBm", max_power))
            }
        };

        ret.extend(additional).into()
    }
}
