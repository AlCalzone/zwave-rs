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
    Any,
}

impl ValueMetadata {
    pub fn any() -> Self {
        Self::Any
    }
}

pub struct CCValueOptions {}

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
///     ValueMetadata::any(), // or any other metadata
///     CCValueOptions {} // or any other value options
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
///     ValueMetadata::any(), // or any other metadata
///     CCValueOptions {} // or any other value options
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
