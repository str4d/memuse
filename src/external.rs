use std::mem;

use crate::DynamicUsage;

#[cfg(feature = "nonempty")]
impl_iterable_dynamic_usage!(nonempty::NonEmpty<T>, |c: &nonempty::NonEmpty<T>| {
    // NonEmpty<T> stores its head element separately from its tail Vec<T>.
    (c.capacity() - 1) * mem::size_of::<T>()
});

mod crossbeam_channel;
