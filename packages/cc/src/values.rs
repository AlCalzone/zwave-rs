use zwave_core::value_id::ValueId;

pub struct CCValue {
    pub id: ValueId,
    pub(crate) is: CCValuePredicate,
    pub metadata: ValueMetadata,
    pub options: CCValueOptions,
}

impl CCValue {
    pub fn is(&self, value_id: &ValueId) -> bool {
        (self.is)(value_id)
    }
}

pub type CCValuePredicate = Box<dyn Fn(&ValueId) -> bool + 'static + Sync + Send>;

pub enum ValueMetadata {
    // Generic value metadata
    Numeric(ValueMetadataNumeric),
    Boolean(ValueMetadataBoolean),
    String(ValueMetadataString),
    // TODO: Color
    Buffer(ValueMetadataBuffer),

    // Primitive arrays
    // TODO: Check if keeping the payload makes sense
    NumericArray(ValueMetadataNumeric),
    BooleanArray(ValueMetadataBoolean),
    StringArray(ValueMetadataString),

    // Z-Wave specific value metadata - we have to distinguish between
    // SET and REPORT values, as they have different semantics
    DurationSet(ValueMetadataCommon<()>),
    DurationReport(ValueMetadataCommon<()>),

    // These are almost like Numeric, but have a defined range...
    LevelSet(ValueMetadataCommon<u8>),
    // ...and an "unknown" state for reported values
    LevelReport(ValueMetadataCommon<u8>),
    // TODO: Consider adding a variant for Levels with min/max set to 0..99

    // BinarySet is identical to Boolean, but is used for consistency with ...
    BinarySet(ValueMetadataCommon<()>),
    // ...the BinaryReport, which has a defined "unknown" state
    BinaryReport(ValueMetadataCommon<()>),
    // TODO: Configuration
}

impl ValueMetadata {
    pub fn duration_set(common: ValueMetadataCommon<()>) -> Self {
        Self::DurationSet(common)
    }
}

pub struct ValueMetadataCommon<T> {
    /// A human-readable name for the value
    pub label: Option<&'static str>,
    /// A detailed description of the value
    pub description: Option<&'static str>,

    /// Whether the value can be read
    pub readable: bool,
    /// Whether the value can be written
    pub writeable: bool,

    /// Human-readable names for some or all of the possible values
    pub states: Option<Vec<(T, &'static str)>>,

    /// Whether a user should be able to manually enter all legal values in the range `min...max` (`true`),
    /// or if only the ones defined in `states` should be selectable in a dropdown (`false`).
    ///
    /// If missing, applications should assume this to be `true` if no `states` are defined and `false` if `states` are defined.
    // FIXME: Set this automatically and remove the Option
    pub allow_manual_entry: Option<bool>,
}

impl<T> Default for ValueMetadataCommon<T> {
    fn default() -> Self {
        Self {
            label: None,
            description: None,
            readable: true,
            writeable: true,
            allow_manual_entry: Some(true),
            states: None,
        }
    }
}

impl<T> ValueMetadataCommon<T> {
    pub fn default_readonly() -> Self {
        Self {
            readable: true,
            writeable: false,
            ..Default::default()
        }
    }

    pub fn default_writeonly() -> Self {
        Self {
            readable: false,
            writeable: true,
            ..Default::default()
        }
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readable = true;
        self.writeable = false;
        self
    }

    pub fn writeonly(mut self) -> Self {
        self.readable = false;
        self.writeable = true;
        self
    }

    pub fn states(mut self, states: Vec<(T, &'static str)>) -> Self {
        self.states = Some(states);
        self
    }

    pub fn allow_manual_entry(mut self, allow_manual_entry: bool) -> Self {
        self.allow_manual_entry = Some(allow_manual_entry);
        self
    }
}

macro_rules! impl_common_metadata_accessors {
    ($t:ty) => {
        pub fn label(mut self, label: &'static str) -> Self {
            self.common = self.common.label(label);
            self
        }

        pub fn description(mut self, description: &'static str) -> Self {
            self.common = self.common.description(description);
            self
        }

        pub fn readonly(mut self) -> Self {
            self.common = self.common.readonly();
            self
        }

        pub fn writeonly(mut self) -> Self {
            self.common = self.common.writeonly();
            self
        }

        pub fn states(mut self, states: Vec<($t, &'static str)>) -> Self {
            self.common = self.common.states(states);
            self
        }

        pub fn allow_manual_entry(mut self, allow_manual_entry: bool) -> Self {
            self.common = self.common.allow_manual_entry(allow_manual_entry);
            self
        }
    };
}

#[derive(Default)]
pub struct ValueMetadataNumeric {
    // In order to keep complexity low, we choose i64 as the only numeric type
    // which should be sufficient to store any Z-Wave number we encounter.
    pub common: ValueMetadataCommon<i64>,

    /// The minimum value that can be assigned to a CC value
    pub min: Option<i64>,
    /// The maximum value that can be assigned to a CC value
    pub max: Option<i64>,
    /// When only certain values between min and max are allowed, this determines the step size
    pub steps: Option<i64>,
    /// The default value
    pub default: Option<i64>,

    /// The unit of the value, e.g. "°C" or "%"
    pub unit: Option<&'static str>,
}

impl ValueMetadataNumeric {
    pub fn common(mut self, common: ValueMetadataCommon<i64>) -> Self {
        self.common = common;
        self
    }

    impl_common_metadata_accessors!(i64);

    pub fn min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn steps(mut self, steps: i64) -> Self {
        self.steps = Some(steps);
        self
    }

    pub fn default_value(mut self, default: i64) -> Self {
        self.default = Some(default);
        self
    }

    pub fn unit(mut self, unit: &'static str) -> Self {
        self.unit = Some(unit);
        self
    }
}

#[derive(Default)]
pub struct ValueMetadataBoolean {
    pub common: ValueMetadataCommon<bool>,

    /// The default value
    pub default: Option<bool>,
}

impl ValueMetadataBoolean {
    pub fn common(mut self, common: ValueMetadataCommon<bool>) -> Self {
        self.common = common;
        self
    }

    impl_common_metadata_accessors!(bool);

    pub fn default_value(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }
}

#[derive(Default)]
pub struct ValueMetadataString {
    pub common: ValueMetadataCommon<()>,

    /// The minimum length this string must have
    pub min_length: Option<usize>,
    /// The maximum length this string can have
    pub max_length: Option<usize>,

    /// The default value
    pub default: Option<String>,
}

impl ValueMetadataString {
    pub fn common(mut self, common: ValueMetadataCommon<()>) -> Self {
        self.common = common;
        self
    }

    impl_common_metadata_accessors!(());

    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
}

#[derive(Default)]
pub struct ValueMetadataBuffer {
    pub common: ValueMetadataCommon<()>,

    /// The minimum length this buffer must have
    pub min_length: Option<usize>,
    /// The maximum length this buffer can have
    pub max_length: Option<usize>,
}

impl ValueMetadataBuffer {
    pub fn common(mut self, common: ValueMetadataCommon<()>) -> Self {
        self.common = common;
        self
    }

    impl_common_metadata_accessors!(());

    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
}

pub struct CCValueOptions {
    /// Whether the CC value is internal. Internal values are not exposed to the user.
    pub internal: bool,

    /// The minimum CC version required for this value to exist.
    pub min_version: u8,

    /// Whether the CC value may exist on endpoints.
    pub supports_endpoints: bool,

    /// Whether this value represents a state (`true`) or a notification/event (`false`)
    pub stateful: bool,
    /// Whether this value should be hidden in logs
    pub secret: bool,

    // FIXME: Add support for dynamic autoCreate
    pub auto_create: bool,
}

impl Default for CCValueOptions {
    fn default() -> Self {
        Self {
            internal: false,
            min_version: 1,
            supports_endpoints: true,
            stateful: true,
            secret: false,
            auto_create: true,
        }
    }
}

impl CCValueOptions {
    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }

    pub fn min_version(mut self, min_version: u8) -> Self {
        self.min_version = min_version;
        self
    }

    pub fn supports_endpoints(mut self, supports_endpoints: bool) -> Self {
        self.supports_endpoints = supports_endpoints;
        self
    }

    pub fn stateful(mut self, stateful: bool) -> Self {
        self.stateful = stateful;
        self
    }

    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    pub fn auto_create(mut self, auto_create: bool) -> Self {
        self.auto_create = auto_create;
        self
    }
}

/// Helper macro to generate value definitions for a CC value with
/// a static `property` and an optional static `property_key`.
///
/// **Note:** This expects an enum with the name `<CCName>CCProperties` to be in scope as well
/// as an impl to convert from the enum to `(u32, Option<u32>)`.
///
/// The given name will be used in `snake_case` as the name of the
/// function that returns the value definition and as-is for accessing the
/// aforementioned enum.
///
/// Usage:
/// ```ignore
/// cc_value_static_property!(
///     CCName, // Must exist in CommandClasses enum
///     ValueName, // Must exist in <CCName>CCProperties enum
///     ValueMetadata::Numeric(ValueMetadataNumeric::default()), // or any other metadata
///     CCValueOptions::default() // or any other value options
/// );
/// ```
///
/// Output:
/// ```ignore
/// pub fn value_name() -> &'static CCValue {
///     // ...
/// }
macro_rules! cc_value_static_property {
    ($cc:ident, $name:ident, $metadata:expr, $options:expr) => {
        paste::paste! {
            pub fn [<$name:snake>]() -> &'static CCValue {
                use std::sync::OnceLock;
                use zwave_core::value_id::ValueId;

                static RET: OnceLock<CCValue> = OnceLock::new();
                RET.get_or_init(|| {
                    let property_and_key: (u32, Option<u32>) = [<$cc CCProperties>]::$name.into();
                    let value_id = ValueId::new(
                        CommandClasses::$cc,
                        property_and_key.0,
                        property_and_key.1,
                    );
                    let is = Box::new(move |id: &ValueId| {
                        (id.property(), id.property_key()) == property_and_key
                    });
                    let metadata = $metadata;
                    let options = $options;

                    CCValue {
                        id: value_id,
                        is,
                        metadata,
                        options,
                    }
                })
            }
        }
    };
}
pub(crate) use cc_value_static_property;

/// Helper macro to generate value definitions for a CC value with
/// a `property` and optional `property_key` that depend on the given method arguments.
///
/// **Note:** This expects an enum with the name `<CCName>CCProperties` to be in scope as well
/// as an impl to convert from the enum to `(u32, Option<u32>)`.
/// The enum must have a tuple-like variant with the given name, which takes the same parameters as the method.
///
/// The given name will be used in `snake_case` as the name of the
/// function that returns the value definition.
///
/// Usage:
/// ```ignore
/// cc_value_dynamic_property!(
///     CCName, // Must exist in CommandClasses enum
///     ValueName, // Must exist in <CCName>CCProperties enum
///     (arg1: u8, arg2: bool), // Must be compatible with the ValueName enum variant
///     ValueMetadata::Numeric(ValueMetadataNumeric::default()), // or any other metadata
///     CCValueOptions::default() // or any other value options
/// );
/// ```
///
/// Output:
/// ```ignore
/// pub fn value_name(arg1: u8, arg2: bool) -> &'static CCValue {
///     // ...
/// }
macro_rules! cc_value_dynamic_property {
    ($cc:ident, $name:ident, ($($param:ident: $type:ty),*), $metadata:expr, $options:expr) => {
        paste::paste! {
            pub fn [<$name:snake>]($($param: $type),*) -> &'static CCValue {
                static RET: OnceLock<CCValue> = OnceLock::new();
                RET.get_or_init(|| {
                    let property_and_key: (u32, Option<u32>) = [<$cc CCProperties>]::$name($($param),*).into();
                    let value_id = ValueId::new(
                        CommandClasses::$cc,
                        property_and_key.0,
                        property_and_key.1,
                    );
                    let is = Box::new(move |id: &ValueId| {
                        (id.property(), id.property_key()) == property_and_key
                    });
                    let metadata = $metadata;
                    let options = $options;

                    CCValue {
                        id: value_id,
                        is,
                        metadata,
                        options,
                    }
                })
            }
        }
    };
}
pub(crate) use cc_value_dynamic_property;
