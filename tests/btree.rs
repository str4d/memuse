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

    for i in 0u16..10 {
        map.insert(i, i);
    }
    let allocated = PEAK_ALLOC.current_usage() - base_mem;

    let (lower, upper) = map.dynamic_usage_bounds();
    assert!(lower <= allocated);
    assert!(allocated <= upper.unwrap());
}
