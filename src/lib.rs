//! Measure dynamic memory usage of your types!

/// Trait for measuring the dynamic memory usage of types.
trait DynamicUsage {
    /// Returns the amount of heap-allocated memory used by this type.
    fn dynamic_usage(&self) -> usize;
}
