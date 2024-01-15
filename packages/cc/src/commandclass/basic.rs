use crate::prelude::*;
use crate::values::*;
use proc_macros::{CCValues, TryFromRepr};
use zwave_core::value_id::ValueIdProperties;
use zwave_core::{cache::CacheValue, prelude::*, value_id::ValueId};

use cookie_factory as cf;
use nom::{
    combinator::{map, opt},
    sequence::tuple,
};
use typed_builder::TypedBuilder;
use zwave_core::encoding::{self, encoders::empty};

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
enum BasicCCProperties {
    CurrentValue = 0x00,
    TargetValue = 0x01,
    Duration = 0x02,
    RestorePrevious = 0x03,
}

impl From<BasicCCProperties> for ValueIdProperties {
    fn from(val: BasicCCProperties) -> Self {
        Self::new(val as u32, None)
    }
}

impl TryFrom<ValueIdProperties> for BasicCCProperties {
    type Error = ();

    fn try_from(val: ValueIdProperties) -> Result<Self, Self::Error> {
        match (Self::try_from(val.property() as u8), val.property_key()) {
            (Ok(prop), None) => Ok(prop),
            _ => Err(()),
        }
    }
}

pub struct BasicCCValues;
impl BasicCCValues {
    cc_value_static_property!(
        Basic,
        CurrentValue,
        ValueMetadata::LevelReport(ValueMetadataCommon::default_readonly().label("Current value")),
        CCValueOptions::default()
    );

    cc_value_static_property!(
        Basic,
        TargetValue,
        ValueMetadata::LevelSet(ValueMetadataCommon::default().label("Target value"),),
        CCValueOptions::default()
    );

    cc_value_static_property!(
        Basic,
        Duration,
        ValueMetadata::DurationReport(
            ValueMetadataCommon::default_readonly().label("Remaining duration"),
        ),
        CCValueOptions::default().min_version(2)
    );

    // Convenience value to restore the previous non-zero value
    cc_value_static_property!(
        Basic,
        RestorePrevious,
        ValueMetadata::Boolean(
            ValueMetadataBoolean::default().common(
                ValueMetadataCommon::default_writeonly()
                    .label("Restore previous value")
                    .states(vec![(true, "Restore"),])
            ),
        ),
        CCValueOptions::default()
    );
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum BasicCCCommand {
    Set = 0x01,
    Get = 0x02,
    Report = 0x03,
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
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

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct BasicCCGet {}

impl CCBase for BasicCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::BasicCCReport(_))
    }
}

impl CCId for BasicCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Basic
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BasicCCCommand::Get as _)
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

#[derive(Debug, Clone, PartialEq, CCValues)]
pub struct BasicCCReport {
    #[cc_value(BasicCCValues::current_value)]
    pub current_value: LevelReport,
    #[cc_value(BasicCCValues::target_value)]
    pub target_value: Option<LevelReport>,
    #[cc_value(BasicCCValues::duration)]
    pub duration: Option<DurationReport>,
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
