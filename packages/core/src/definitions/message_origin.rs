/// Where a serialized message originates from, to distinguish how certain messages need to be deserialized
pub enum MessageOrigin {
    Controller,
    Host,
}
