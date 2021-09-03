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
impl_no_dynamic_usage!(&str);

impl DynamicUsage for String {
    fn dynamic_usage(&self) -> usize {
        self.capacity()
    }
}

impl<T> DynamicUsage for Option<T>
where
    T: DynamicUsage,
{
    fn dynamic_usage(&self) -> usize {
        self.as_ref().map(DynamicUsage::dynamic_usage).unwrap_or(0)
    }
}

impl<T> DynamicUsage for &[T]
where
    T: DynamicUsage,
{
    fn dynamic_usage(&self) -> usize {
        self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }
}

impl<T> DynamicUsage for Vec<T>
where
    T: DynamicUsage,
{
    fn dynamic_usage(&self) -> usize {
        self.capacity() * mem::size_of::<T>() + self.as_slice().dynamic_usage()
    }
}

#[cfg(feature = "nonempty")]
impl<T> DynamicUsage for nonempty::NonEmpty<T>
where
    T: DynamicUsage,
{
    fn dynamic_usage(&self) -> usize {
        // NonEmpty<T> stores its head element separately from its tail Vec<T>.
        (self.capacity() - 1) * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_types() {
        assert_eq!(129u8.dynamic_usage(), 0);
        assert_eq!(3i128.dynamic_usage(), 0);
        assert_eq!(7.0f32.dynamic_usage(), 0);
        assert_eq!("foobar".dynamic_usage(), 0);
    }

    #[test]
    fn string() {
        assert_eq!(String::new().dynamic_usage(), 0);
        assert_eq!("foobar".to_string().dynamic_usage(), 6);
    }

    #[test]
    fn option() {
        let a: Option<Vec<u8>> = None;
        let b: Option<Vec<u8>> = Some(vec![7u8; 4]);
        assert_eq!(a.dynamic_usage(), 0);
        assert_eq!(b.dynamic_usage(), 4);
    }

    #[test]
    fn vec() {
        let capacity = 7;
        let mut a = Vec::with_capacity(capacity);
        a.push(42u64);
        assert_eq!(a.dynamic_usage(), capacity * mem::size_of::<u64>());
    }

    #[cfg(feature = "nonempty")]
    #[test]
    fn nonempty() {
        let a = nonempty::NonEmpty::new(42);
        assert_eq!(a.dynamic_usage(), 0);

        const CAPACITY: usize = 7;
        let b = nonempty::NonEmpty::from_slice(&[27u128; CAPACITY]).unwrap();
        assert_eq!(b.dynamic_usage(), (CAPACITY - 1) * mem::size_of::<u128>());
    }
}
