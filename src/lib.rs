//! Measure dynamic memory usage of your types!

#![forbid(unsafe_code)]
// Catch documentation errors caused by code changes.
#![deny(broken_intra_doc_links)]

use core::mem;
use std::collections::{BinaryHeap, LinkedList, VecDeque};

mod hash;

/// Trait for measuring the dynamic memory usage of types.
pub trait DynamicUsage {
    /// Returns a best estimate of the amount of heap-allocated memory used by this type.
    ///
    /// For most types, this will return an exact value. However, for types that use a
    /// complex allocation strategy (such as a `BTreeMap`), `memuse` cannot provide an
    /// exact heap allocation value, as it does not have access to the internal details
    /// and can only infer allocations from observable properties (such as the number of
    /// elements in a collection, or constants extracted from the implementation of the
    /// type). In those cases, this method returns a "best estimate" inferred from the
    /// implemented behaviour of the type. As more crates implement this trait themselves,
    /// the estimates will become more precise.
    ///
    /// The value returned by this method will always fall between the bounds returned by
    /// [`DynamicUsage::dynamic_usage_bounds`]:
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use memuse::DynamicUsage;
    ///
    /// let a: HashMap<u8, u64> = HashMap::with_capacity(27);
    /// let usage = a.dynamic_usage();
    /// let (lower, upper) = a.dynamic_usage_bounds();
    ///
    /// assert!(lower <= usage);
    /// if let Some(upper) = upper {
    ///     assert!(usage <= upper);
    /// }
    /// ```
    fn dynamic_usage(&self) -> usize;

    /// Returns the lower and upper bounds on the amount of heap-allocated memory used by
    /// this type.
    ///
    /// The lower bound is always precise; a type cannot allocate fewer than zero bytes,
    /// and a collection cannot allocate fewer than the number of bytes required to store
    /// the entries it holds.
    ///
    /// The upper bound is only present if some property of the type ensures that its
    /// allocations do not exceed the bound, and is `None` otherwise (to indicate an
    /// unlimited upper bound).
    ///
    /// If the type's allocated memory is precisely known, then the lower and upper bounds
    /// will be equal.
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>);
}

/// Marker trait for types that do not use heap-allocated memory.
pub trait NoDynamicUsage {}

impl<T: NoDynamicUsage> DynamicUsage for T {
    #[inline(always)]
    fn dynamic_usage(&self) -> usize {
        0
    }

    #[inline(always)]
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        (0, Some(0))
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

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        let usage = self.capacity();
        (usage, Some(usage))
    }
}

impl<T: DynamicUsage> DynamicUsage for Option<T> {
    fn dynamic_usage(&self) -> usize {
        self.as_ref().map(DynamicUsage::dynamic_usage).unwrap_or(0)
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.as_ref()
            .map(DynamicUsage::dynamic_usage_bounds)
            .unwrap_or((0, Some(0)))
    }
}

//
// Collections
//

/// Returns the dynamic usage bounds for this iterable.
fn iter_usage_bounds<'a, T: DynamicUsage + 'a, I: Iterator<Item = &'a T>>(
    base_usage: usize,
    i: I,
) -> (usize, Option<usize>) {
    let (lower, upper) = i.map(DynamicUsage::dynamic_usage_bounds).fold(
        (0, Some(0)),
        |(acc_lower, acc_upper), (lower, upper)| {
            (acc_lower + lower, acc_upper.zip(upper).map(|(a, b)| a + b))
        },
    );
    (base_usage + lower, upper.map(|u| base_usage + u))
}

impl<T: DynamicUsage> DynamicUsage for &[T] {
    fn dynamic_usage(&self) -> usize {
        self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        iter_usage_bounds(0, self.iter())
    }
}

impl<T: DynamicUsage> DynamicUsage for Vec<T> {
    fn dynamic_usage(&self) -> usize {
        self.capacity() * mem::size_of::<T>() + self.as_slice().dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        iter_usage_bounds(self.capacity() * mem::size_of::<T>(), self.iter())
    }
}

impl<T: DynamicUsage> DynamicUsage for BinaryHeap<T> {
    fn dynamic_usage(&self) -> usize {
        // BinaryHeap<T> is a wrapper around Vec<T>
        self.capacity() * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        iter_usage_bounds(self.capacity() * mem::size_of::<T>(), self.iter())
    }
}

impl<T: DynamicUsage> DynamicUsage for LinkedList<T> {
    fn dynamic_usage(&self) -> usize {
        self.len() * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        iter_usage_bounds(self.len() * mem::size_of::<T>(), self.iter())
    }
}

impl<T: DynamicUsage> DynamicUsage for VecDeque<T> {
    fn dynamic_usage(&self) -> usize {
        // +1 since the ringbuffer always leaves one space empty.
        (self.capacity() + 1) * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        iter_usage_bounds((self.capacity() + 1) * mem::size_of::<T>(), self.iter())
    }
}

#[cfg(feature = "nonempty")]
impl<T: DynamicUsage> DynamicUsage for nonempty::NonEmpty<T> {
    fn dynamic_usage(&self) -> usize {
        // NonEmpty<T> stores its head element separately from its tail Vec<T>.
        (self.capacity() - 1) * mem::size_of::<T>()
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        iter_usage_bounds((self.capacity() - 1) * mem::size_of::<T>(), self.iter())
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

        assert_eq!(129u8.dynamic_usage_bounds(), (0, Some(0)));
        assert_eq!(3i128.dynamic_usage_bounds(), (0, Some(0)));
        assert_eq!(7.0f32.dynamic_usage_bounds(), (0, Some(0)));
        assert_eq!("foobar".dynamic_usage_bounds(), (0, Some(0)));
    }

    #[test]
    fn string() {
        assert_eq!(String::new().dynamic_usage(), 0);
        assert_eq!("foobar".to_string().dynamic_usage(), 6);

        assert_eq!(String::new().dynamic_usage_bounds(), (0, Some(0)));
        assert_eq!("foobar".to_string().dynamic_usage_bounds(), (6, Some(6)));
    }

    #[test]
    fn option() {
        let a: Option<Vec<u8>> = None;
        let b: Option<Vec<u8>> = Some(vec![7u8; 4]);
        assert_eq!(a.dynamic_usage(), 0);
        assert_eq!(a.dynamic_usage_bounds(), (0, Some(0)));
        assert_eq!(b.dynamic_usage(), 4);
        assert_eq!(b.dynamic_usage_bounds(), (4, Some(4)));
    }

    #[test]
    fn vec() {
        let capacity = 7;
        let mut a = Vec::with_capacity(capacity);
        a.push(42u64);

        let expected = capacity * mem::size_of::<u64>();
        assert_eq!(a.dynamic_usage(), expected);
        assert_eq!(a.dynamic_usage_bounds(), (expected, Some(expected)));
    }

    #[cfg(feature = "nonempty")]
    #[test]
    fn nonempty() {
        let a = nonempty::NonEmpty::new(42);
        assert_eq!(a.dynamic_usage(), 0);
        assert_eq!(a.dynamic_usage_bounds(), (0, Some(0)));

        const CAPACITY: usize = 7;
        let b = nonempty::NonEmpty::from_slice(&[27u128; CAPACITY]).unwrap();

        let expected = (CAPACITY - 1) * mem::size_of::<u128>();
        assert_eq!(b.dynamic_usage(), expected);
        assert_eq!(b.dynamic_usage_bounds(), (expected, Some(expected)));
    }
}
