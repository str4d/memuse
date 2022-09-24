use std::collections::BTreeMap;

use memuse::DynamicUsage;
use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

#[test]
fn test_bounds() {
    let base_mem = PEAK_ALLOC.current_usage();

    // No allocations for a new tree.
    let mut map = BTreeMap::new();
    assert_eq!(PEAK_ALLOC.current_usage(), base_mem);

    // Insert 10 items.
    for i in 0u16..10 {
        map.insert(i, i);
    }
    let allocated = PEAK_ALLOC.current_usage() - base_mem;

    // The actual allocations should fall within the calculated bounds.
    let (lower, upper) = map.dynamic_usage_bounds();
    assert!(lower <= allocated);
    assert!(allocated <= upper.unwrap());

    // No allocations by test.
    assert_eq!(PEAK_ALLOC.current_usage() - base_mem, allocated);

    // Insert 1000 more items.
    for i in 0u16..1000 {
        map.insert(10 + i, i);
    }
    let allocated = PEAK_ALLOC.current_usage() - base_mem;

    // The actual allocations should fall within the calculated bounds.
    let (lower, upper) = map.dynamic_usage_bounds();
    assert!(lower <= allocated);
    assert!(allocated <= upper.unwrap());

    // No allocations by test.
    assert_eq!(PEAK_ALLOC.current_usage() - base_mem, allocated);
}
