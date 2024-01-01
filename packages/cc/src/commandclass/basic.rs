use std::sync::OnceLock;

use crate::{
    prelude::*,
    values::{cc_value_static_property, CCValue, CCValueOptions, ValueMetadata},
};
use zwave_core::{cache::CacheValue, prelude::*, value_id::ValueId};

use cookie_factory as cf;
use derive_try_from_primitive::TryFromPrimitive;
use nom::{
    combinator::{map, opt},
    sequence::tuple,
};
use typed_builder::TypedBuilder;
use zwave_core::encoding::{self, encoders::empty};

enum BasicCCProperties {
    CurrentValue = 0x00,
    TargetValue = 0x01,
    Duration = 0x02,
}

impl From<BasicCCProperties> for (u32, Option<u32>) {
    fn from(val: BasicCCProperties) -> Self {
        (val as u32, None)
    }
}

pub struct BasicCCValues;
impl BasicCCValues {
    cc_value_static_property!(Basic, CurrentValue, ValueMetadata::any(), CCValueOptions {});

    cc_value_static_property!(Basic, TargetValue, ValueMetadata::any(), CCValueOptions {});

    cc_value_static_property!(Basic, Duration, ValueMetadata::any(), CCValueOptions {});
}

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

impl CCValues for BasicCCSet {}

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

impl CCValues for BasicCCGet {}

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
    pub duration: Option<DurationReport>,
}

impl CCBase for BasicCCReport {}

// FIXME: Create a derive macro for this
impl CCValues for BasicCCReport {
    fn to_values(&self) -> Vec<(ValueId, CacheValue)> {
        let mut ret = vec![(
            BasicCCValues::current_value().id,
            CacheValue::from(self.current_value),
        )];

        if let Some(target_value) = self.target_value {
            ret.push((
                BasicCCValues::target_value().id,
                CacheValue::from(target_value),
            ));
        }

        if let Some(duration) = self.duration {
            ret.push((BasicCCValues::duration().id, CacheValue::from(duration)));
        }

        ret
    }
}

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
        let (i, (target_value, duration)) = map(
            opt(tuple((LevelReport::parse, DurationReport::parse))),
            |x| x.unzip(),
        )(i)?;

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
        use cf::sequence::tuple;

        let serialize_target_and_duration = move |out| match self.target_value {
            Some(target_value) => tuple((
                target_value.serialize(),
                self.duration.unwrap_or_default().serialize(),
            ))(out),
            None => empty()(out),
        };

        tuple((
            self.current_value.serialize(),
            serialize_target_and_duration,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_cc_values() {
        let current_value = BasicCCValues::current_value();
        assert!(current_value.is(&current_value.id));

        let target_value = BasicCCValues::target_value();
        assert!(target_value.is(&target_value.id));

        let duration = BasicCCValues::duration();
        assert!(duration.is(&duration.id));
    }
}
