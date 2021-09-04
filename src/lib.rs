//! Measure dynamic memory usage of your types!

#![forbid(unsafe_code)]
// Catch documentation errors caused by code changes.
#![deny(broken_intra_doc_links)]

use core::mem;
use std::collections::{BinaryHeap, LinkedList, VecDeque};

/// Trait for measuring the dynamic memory usage of types.
pub trait DynamicUsage {
    /// Returns the amount of heap-allocated memory used by this type.
    fn dynamic_usage(&self) -> usize;
}

/// Marker trait for types that do not use heap-allocated memory.
pub trait NoDynamicUsage {}

impl<T: NoDynamicUsage> DynamicUsage for T {
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

impl<T: DynamicUsage> DynamicUsage for Option<T> {
    fn dynamic_usage(&self) -> usize {
        self.as_ref().map(DynamicUsage::dynamic_usage).unwrap_or(0)
    }
}

//
// Collections
//

impl<T: DynamicUsage> DynamicUsage for &[T] {
    fn dynamic_usage(&self) -> usize {
        self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }
}

impl<T: DynamicUsage> DynamicUsage for Vec<T> {
    fn dynamic_usage(&self) -> usize {
        self.capacity() * mem::size_of::<T>() + self.as_slice().dynamic_usage()
    }
}

impl<T: DynamicUsage> DynamicUsage for BinaryHeap<T> {
    fn dynamic_usage(&self) -> usize {
        // BinaryHeap<T> is a wrapper around Vec<T>
        self.capacity() * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }
}

impl<T: DynamicUsage> DynamicUsage for LinkedList<T> {
    fn dynamic_usage(&self) -> usize {
        self.len() * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }
}

impl<T: DynamicUsage> DynamicUsage for VecDeque<T> {
    fn dynamic_usage(&self) -> usize {
        // +1 since the ringbuffer always leaves one space empty.
        (self.capacity() + 1) * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }
}

#[cfg(feature = "nonempty")]
impl<T: DynamicUsage> DynamicUsage for nonempty::NonEmpty<T> {
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
