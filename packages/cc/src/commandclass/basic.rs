use crate::prelude::*;
use derive_try_from_primitive::TryFromPrimitive;
use nom::combinator::opt;
use nom::sequence::tuple;
use typed_builder::TypedBuilder;
use zwave_core::encoding::encoders::empty;
use zwave_core::prelude::*;

use cookie_factory as cf;
use nom::{combinator::map, number::complete::be_u8};
use zwave_core::definitions::CommandClasses;
use zwave_core::encoding::{self};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum BasicCCCommand {
    Set = 0x01,
    Get = 0x02,
    Report = 0x03,
}

#[derive(Debug, Clone, PartialEq, TypedBuilder)]
pub struct BasicCCSet {
    pub target_value: LevelSet,
}

impl CCBase for BasicCCSet {}

impl CCId for BasicCCSet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Basic
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BasicCCCommand::Set as _)
    }
}

impl CCRequest for BasicCCSet {
    fn expects_response(&self) -> bool {
        false
    }

    fn test_response(&self, _response: &CC) -> bool {
        false
    }
}

impl CCParsable for BasicCCSet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, target_value) = LevelSet::parse(i)?;

        Ok((i, Self { target_value }))
    }
}

impl CCSerializable for BasicCCSet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        self.target_value.serialize()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct BasicCCGet {}

impl CCBase for BasicCCGet {}

impl CCId for BasicCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Basic
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BasicCCCommand::Get as _)
    }
}

impl CCRequest for BasicCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::BasicCCReport(_))
    }
}

impl CCParsable for BasicCCGet {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl CCSerializable for BasicCCGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicCCReport {
    pub current_value: LevelReport,
    pub target_value: Option<LevelReport>,
    pub duration: Option<u8>, // FIXME: This should be its own struct/enum
}

impl CCBase for BasicCCReport {}

impl CCId for BasicCCReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Basic
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BasicCCCommand::Report as _)
    }
}

impl CCParsable for BasicCCReport {
    fn parse<'a>(i: encoding::Input<'a>, _ctx: &CCParsingContext) -> ParseResult<'a, Self> {
        let (i, current_value) = LevelReport::parse(i)?;
        let (i, (target_value, duration)) =
            map(opt(tuple((LevelReport::parse, be_u8))), |x| x.unzip())(i)?;

        Ok((
            i,
            Self {
                current_value,
                target_value,
                duration,
            },
        ))
    }
}

impl CCSerializable for BasicCCReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        // FIXME: Only include target_value and duration in V2 of the CC
        use cf::bytes::be_u8;
        use cf::sequence::tuple;

        let serialize_target_and_duration = move |out| match self.target_value {
            Some(target_value) => tuple((
                target_value.serialize(),
                be_u8(self.duration.unwrap_or(0xfe)),
            ))(out),
            None => empty()(out),
        };

        tuple((
            self.current_value.serialize(),
            serialize_target_and_duration,
        ))
    }
}
