//! Measure dynamic memory usage of your types!

use core::mem;

/// Trait for measuring the dynamic memory usage of types.
trait DynamicUsage {
    /// Returns the amount of heap-allocated memory used by this type.
    fn dynamic_usage(&self) -> usize;
}

/// Marker trait for types that do not use heap-allocated memory.
trait NoDynamicUsage {}

impl<T> DynamicUsage for T
where
    T: NoDynamicUsage,
{
    #[inline(always)]
    fn dynamic_usage(&self) -> usize {
        0
    }
}

macro_rules! impl_no_dynamic_usage {
    ($($type:ty),+) => {
        $(impl NoDynamicUsage for $type {})+
    };
}

impl_no_dynamic_usage!(i8, i16, i32, i64, i128, isize);
impl_no_dynamic_usage!(u8, u16, u32, u64, u128, usize);
impl_no_dynamic_usage!(f32, f64, char, bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_types() {
        assert_eq!(129u8.dynamic_usage(), 0);
        assert_eq!(3i128.dynamic_usage(), 0);
        assert_eq!(7.0f32.dynamic_usage(), 0);
    }
}
