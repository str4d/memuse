use std::mem;
use std::sync::atomic::AtomicUsize;

use crossbeam_channel::{Receiver, Sender};

use crate::DynamicUsage;

enum ChannelFlavor {
    /// Bounded channel based on a preallocated array.
    Array,
    /// Unbounded channel implemented as a linked list.
    List,
    /// Zero-capacity channel.
    Zero,
    /// The after flavor.
    At,
    /// The tick flavor.
    Tick,
    /// The never flavor.
    Never,
}

impl ChannelFlavor {
    fn guess<T>(rx: &Receiver<T>) -> Self {
        match rx.capacity() {
            // Could be Zero or Never.
            Some(0) => Self::Zero,
            // Could be Array, At, or Tick.
            Some(1) => Self::Array,
            // Array.
            Some(_) => Self::Array,
            // List.
            None => Self::List,
        }
    }
}

impl<T: DynamicUsage> DynamicUsage for Sender<T> {
    #[inline(always)]
    fn dynamic_usage(&self) -> usize {
        // We count the memory usage of items in the channel on the receiver side.
        0
    }

    #[inline(always)]
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<T: DynamicUsage> DynamicUsage for Receiver<T> {
    fn dynamic_usage(&self) -> usize {
        // We count the memory usage of items in the channel on the receiver side.
        match ChannelFlavor::guess(self) {
            ChannelFlavor::List => {
                // The items in the channel are stored as a linked list. Memory for the
                // list is allocated in blocks of 31 items.
                const ITEMS_PER_BLOCK: usize = 31;
                let num_items = self.len();
                let num_blocks = (num_items + ITEMS_PER_BLOCK - 1) / ITEMS_PER_BLOCK;

                // The structure of a block is:
                // - A pointer to the next block.
                // - For each slot in the block:
                //   - Space for an item.
                //   - The state of the slot, stored as an AtomicUsize.
                const PTR_SIZE: usize = mem::size_of::<usize>();
                let item_size = mem::size_of::<T>();
                const ATOMIC_USIZE_SIZE: usize = mem::size_of::<AtomicUsize>();
                let block_size = PTR_SIZE + ITEMS_PER_BLOCK * (item_size + ATOMIC_USIZE_SIZE);

                num_blocks * block_size
            }
        }
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        // TODO: Specialize
        let usage = self.dynamic_usage();
        (usage, Some(usage))
    }
}
