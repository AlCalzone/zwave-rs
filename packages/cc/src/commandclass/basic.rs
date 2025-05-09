use crate::prelude::*;
use crate::values::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use typed_builder::TypedBuilder;
use zwave_core::parse::combinators::{map, opt};
use zwave_core::prelude::*;
use zwave_core::{
    cache::CacheValue,
    value_id::{ValueId, ValueIdProperties},
};

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
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let target_value = LevelSet::parse(i)?;

        Ok(Self { target_value })
    }
}

impl SerializableWith<&CCEncodingContext> for BasicCCSet {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        self.target_value.serialize(output)
    }
}

impl ToLogPayload for BasicCCSet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("target value", self.target_value.to_string())
            .into()
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
    fn parse(_i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for BasicCCGet {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for BasicCCGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
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
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let current_value = LevelReport::parse(i)?;
        let (target_value, duration) = map(opt((LevelReport::parse, DurationReport::parse)), |x| {
            x.unzip()
        })
        .parse(i)?;

        Ok(Self {
            current_value,
            target_value,
            duration,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for BasicCCReport {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        self.current_value.serialize(output);

        if let Some(ref target_value) = self.target_value {
            target_value.serialize(output);
            self.duration.unwrap_or_default().serialize(output);
        }
    }
}

impl ToLogPayload for BasicCCReport {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret =
            LogPayloadDict::new().with_entry("current value", self.current_value.to_string());
        if let Some(target_value) = self.target_value {
            ret = ret.with_entry("target value", target_value.to_string());
        }
        if let Some(duration) = self.duration {
            ret = ret.with_entry("duration", duration.to_string());
        }

        ret.into()
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
