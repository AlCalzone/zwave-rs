use zwave_core::value_id::ValueId;

pub struct CCValue {
    pub id: ValueId,
    pub is: CCValuePredicate,
    pub metadata: ValueMetadata,
    pub options: CCValueOptions,
}

pub type CCValuePredicate = Box<dyn Fn(&ValueId) -> bool + 'static + Sync + Send>;

pub enum ValueMetadata {
    Any
}

impl ValueMetadata {
    pub fn any() -> Self {
        Self::Any
    }
}

pub struct CCValueOptions {}
