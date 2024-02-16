use crate::prelude::*;
use crate::values::*;
use bytes::Bytes;
use cookie_factory as cf;
use proc_macros::{CCValues, TryFromRepr};
use typed_builder::TypedBuilder;
use zwave_core::cache::CacheValue;
use zwave_core::encoding::encoders::empty;
use zwave_core::munch::combinators::{map, opt};
use zwave_core::prelude::*;
use zwave_core::value_id::{ValueId, ValueIdProperties};

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
// FIXME: Create derive macro to implement
// From<...> for ValueIdProperties and TryFrom<ValueIdProperties>
// for static-only CC properties
enum BinarySwitchCCProperties {
    CurrentValue = 0x00,
    TargetValue = 0x01,
    Duration = 0x02,
}

impl From<BinarySwitchCCProperties> for ValueIdProperties {
    fn from(val: BinarySwitchCCProperties) -> Self {
        Self::new(val as u32, None)
    }
}

impl TryFrom<ValueIdProperties> for BinarySwitchCCProperties {
    type Error = ();

    fn try_from(val: ValueIdProperties) -> Result<Self, Self::Error> {
        match (Self::try_from(val.property() as u8), val.property_key()) {
            (Ok(prop), None) => Ok(prop),
            _ => Err(()),
        }
    }
}

pub struct BinarySwitchCCValues;
impl BinarySwitchCCValues {
    cc_value_static_property!(
        BinarySwitch,
        CurrentValue,
        ValueMetadata::Boolean(
            ValueMetadataBoolean::default()
                .readonly()
                .label("Current value")
        ),
        CCValueOptions::default()
    );

    cc_value_static_property!(
        BinarySwitch,
        TargetValue,
        ValueMetadata::Boolean(
            ValueMetadataBoolean::default().label("Target value") // TODO: valueChangeOptions: ["transitionDuration"]
        ),
        CCValueOptions::default()
    );

    cc_value_static_property!(
        BinarySwitch,
        Duration,
        ValueMetadata::DurationReport(
            ValueMetadataCommon::default_readonly().label("Remaining duration"),
        ),
        CCValueOptions::default().min_version(2)
    );
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum BinarySwitchCCCommand {
    Set = 0x01,
    Get = 0x02,
    Report = 0x03,
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct BinarySwitchCCSet {
    pub target_value: BinarySet,
    #[builder(default, setter(into))]
    pub duration: Option<DurationSet>,
}

impl CCBase for BinarySwitchCCSet {}

impl CCId for BinarySwitchCCSet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::BinarySwitch
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BinarySwitchCCCommand::Set as _)
    }
}

impl CCParsable for BinarySwitchCCSet {
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::munch::ParseResult<Self> {
        let target_value = BinarySet::parse(i)?;
        let duration = opt(DurationSet::parse).parse(i)?;

        Ok(Self {
            target_value,
            duration,
        })
    }
}

impl CCSerializable for BinarySwitchCCSet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::sequence::tuple;
        tuple((self.target_value.serialize(), self.duration.serialize()))
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct BinarySwitchCCGet {}

impl CCBase for BinarySwitchCCGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::BinarySwitchCCReport(_))
    }
}

impl CCId for BinarySwitchCCGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::BinarySwitch
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BinarySwitchCCCommand::Get as _)
    }
}

impl CCParsable for BinarySwitchCCGet {
    fn parse(_i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::munch::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl CCSerializable for BinarySwitchCCGet {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct BinarySwitchCCReport {
    #[cc_value(BinarySwitchCCValues::current_value)]
    pub current_value: BinaryReport,
    #[cc_value(BinarySwitchCCValues::target_value)]
    pub target_value: Option<BinaryReport>,
    #[cc_value(BinarySwitchCCValues::duration)]
    pub duration: Option<DurationReport>,
}

impl CCBase for BinarySwitchCCReport {}

impl CCId for BinarySwitchCCReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::BinarySwitch
    }

    fn cc_command(&self) -> Option<u8> {
        Some(BinarySwitchCCCommand::Report as _)
    }
}

impl CCParsable for BinarySwitchCCReport {
    fn parse(i: &mut Bytes, _ctx: &CCParsingContext) -> zwave_core::munch::ParseResult<Self> {
        let current_value = BinaryReport::parse(i)?;
        let (target_value, duration) =
            map(opt((BinaryReport::parse, DurationReport::parse)), |x| {
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

impl CCSerializable for BinarySwitchCCReport {
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
