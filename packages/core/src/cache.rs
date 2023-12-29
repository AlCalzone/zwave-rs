use crate::prelude::*;
use paste::paste;

/// Defines the possible values that can be stored in the cache
#[derive(Debug, Clone)]
pub enum CacheValue {
    // Primitives
    Bool(bool),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Float(f32),
    String(String),
    Buffer(Vec<u8>),
    // Z-Wave specific
    DurationSet(DurationSet),
    DurationReport(DurationReport),
    LevelSet(LevelSet),
    LevelReport(LevelReport),
    BinarySet(BinarySet),
    BinaryReport(BinaryReport),
}

pub trait Cache<TKey> {
    fn read(&self, key: &TKey) -> Option<CacheValue>;
    fn write(&mut self, key: &TKey, value: CacheValue);
    fn delete(&mut self, key: &TKey);
}

pub trait CacheExt<TKey> {
    fn read_bool(&self, key: &TKey) -> Option<bool>;
    fn read_u8(&self, key: &TKey) -> Option<u8>;
    fn read_u16(&self, key: &TKey) -> Option<u16>;
    fn read_u32(&self, key: &TKey) -> Option<u32>;
    fn read_i8(&self, key: &TKey) -> Option<i8>;
    fn read_i16(&self, key: &TKey) -> Option<i16>;
    fn read_i32(&self, key: &TKey) -> Option<i32>;
    fn read_f32(&self, key: &TKey) -> Option<f32>;
    fn read_string(&self, key: &TKey) -> Option<String>;
    fn read_buffer(&self, key: &TKey) -> Option<Vec<u8>>;
    fn read_duration_set(&self, key: &TKey) -> Option<DurationSet>;
    fn read_duration_report(&self, key: &TKey) -> Option<DurationReport>;
    fn read_level_set(&self, key: &TKey) -> Option<LevelSet>;
    fn read_level_report(&self, key: &TKey) -> Option<LevelReport>;
    fn read_binary_set(&self, key: &TKey) -> Option<BinarySet>;
    fn read_binary_report(&self, key: &TKey) -> Option<BinaryReport>;

    fn write_bool(&mut self, key: &TKey, value: bool);
    fn write_u8(&mut self, key: &TKey, value: u8);
    fn write_u16(&mut self, key: &TKey, value: u16);
    fn write_u32(&mut self, key: &TKey, value: u32);
    fn write_i8(&mut self, key: &TKey, value: i8);
    fn write_i16(&mut self, key: &TKey, value: i16);
    fn write_i32(&mut self, key: &TKey, value: i32);
    fn write_f32(&mut self, key: &TKey, value: f32);
    fn write_string(&mut self, key: &TKey, value: String);
    fn write_buffer(&mut self, key: &TKey, value: Vec<u8>);
    fn write_duration_set(&mut self, key: &TKey, value: DurationSet);
    fn write_duration_report(&mut self, key: &TKey, value: DurationReport);
    fn write_level_set(&mut self, key: &TKey, value: LevelSet);
    fn write_level_report(&mut self, key: &TKey, value: LevelReport);
    fn write_binary_set(&mut self, key: &TKey, value: BinarySet);
    fn write_binary_report(&mut self, key: &TKey, value: BinaryReport);
}

macro_rules! impl_cache_read_write {
    ($name:ident, $ty:ty, $variant:ident) => {
        paste! {
            fn [<read_ $name>](&self, key: &TKey) -> Option<$ty> {
                match self.read(key) {
                    Some(CacheValue::$variant(value)) => Some(value),
                    _ => None,
                }
            }

            fn [<write_ $name>](&mut self, key: &TKey, value: $ty) {
                self.write(key, CacheValue::$variant(value));
            }
        }
    };
}

impl<TKey, T> CacheExt<TKey> for T
where
    T: Cache<TKey>,
{
    impl_cache_read_write!(bool, bool, Bool);
    impl_cache_read_write!(u8, u8, UInt8);
    impl_cache_read_write!(u16, u16, UInt16);
    impl_cache_read_write!(u32, u32, UInt32);
    impl_cache_read_write!(i8, i8, Int8);
    impl_cache_read_write!(i16, i16, Int16);
    impl_cache_read_write!(i32, i32, Int32);
    impl_cache_read_write!(f32, f32, Float);
    impl_cache_read_write!(string, String, String);
    impl_cache_read_write!(buffer, Vec<u8>, Buffer);
    impl_cache_read_write!(duration_set, DurationSet, DurationSet);
    impl_cache_read_write!(duration_report, DurationReport, DurationReport);
    impl_cache_read_write!(level_set, LevelSet, LevelSet);
    impl_cache_read_write!(level_report, LevelReport, LevelReport);
    impl_cache_read_write!(binary_set, BinarySet, BinarySet);
    impl_cache_read_write!(binary_report, BinaryReport, BinaryReport);
}
