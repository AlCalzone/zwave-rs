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
macro_rules! cc_value_static_property {
    ($name:ident, $cc:ident, $prop:ident, $metadata:expr, $options:expr) => {
        paste::paste! {
            pub fn $name() -> &'static CCValue {
                static RET: OnceLock<CCValue> = OnceLock::new();
                RET.get_or_init(|| {
                    let property_and_key: (u32, Option<u32>) = [<$cc CCProperties>]::$prop.into();
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
