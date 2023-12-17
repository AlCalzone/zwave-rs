/// Where a serialized message originates from, to distinguish how certain messages need to be deserialized
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageOrigin {
    Controller,
    Host,
}
