//! `DynamicUsage` impls for `BTreeMap` and `BTreeSet`.

use std::{
    collections::{BTreeMap, BTreeSet},
    mem::{self, MaybeUninit},
    ptr::NonNull,
};

use crate::DynamicUsage;

// Constants and structures are sourced from here:
//   https://github.com/rust-lang/rust/blob/03c775c95596cbd92f2b1e8ca98e7addfa3eade2/library/alloc/src/collections/btree/node.rs

const B: usize = 6;
const CAPACITY: usize = 2 * B - 1;

#[allow(dead_code)]
struct LeafNode<K, V> {
    parent: Option<NonNull<InternalNode<K, V>>>,
    parent_idx: MaybeUninit<u16>,
    len: u16,
    keys: [MaybeUninit<K>; CAPACITY],
    vals: [MaybeUninit<V>; CAPACITY],
}

#[allow(dead_code)]
struct InternalNode<K, V> {
    data: LeafNode<K, V>,
    edges: [MaybeUninit<BoxedNode<K, V>>; 2 * B],
}

type BoxedNode<K, V> = NonNull<LeafNode<K, V>>;

fn btree_dynamic_usage_bounds<K, V>(entries: usize) -> (usize, usize) {
    // For a classic B-tree, the range of possible heights is:
    //   h_min = ceil(log_m(entries + 1)) - 1
    //   h_max = floor(log_d((entries + 1) / 2))
    // where:
    // - m: maximum number of children for any node (2 * B).
    // - d: minimum number of children for internal nodes (B).
    //
    // The number of nodes in a max-filled B-tree of height h is given by the series
    //   n = 1 + m + m^2 + ... + m^(h-1) = (m^h - 1)/(m - 1)
    //     => (m^(h-1) - 1)/(m - 1) inner nodes
    //         m^(h-1) leaf nodes
    //
    // while the number of nodes for a min-filled tree is given by the series
    //   n = 1 + 2 + 2d + 2d^2 + ... + 2d^(h-2) = 1 + 2*(d^(h-1) - 1)/(d - 1)
    //     => 1 + 2*(d^(h-2) - 1)/(d - 1) inner nodes
    //        2d^(h-2) leaf nodes
    //
    // BTreeMap also relies on several invariants:
    // - Trees must have uniform depth/height. This means that every path down to a leaf
    //   from a given node has exactly the same length, and means we can treat the tree as
    //   exactly balanced at its bounds.
    // - A node of length n has n keys, n values, and n + 1 edges. This means we can get
    //   exact bounds on the memory usage by setting n to either d - 1 or m - 1 for all
    //   nodes.

    let m = 2 * B;
    let d = B;

    let inner_size = mem::size_of::<InternalNode<K, V>>();
    let leaf_size = mem::size_of::<LeafNode<K, V>>();

    // Lower bound:
    let h_min = ((entries + 1) as f64).log(m as f64).ceil() as u32 - 1;
    let (lower_inner, lower_leaf) = match h_min {
        0 => (0, 1),
        _ => ((m.pow(h_min - 1) - 1) / (m - 1), m.pow(h_min - 1)),
    };

    // Upper bound:
    let h_max = (((entries + 1) as f64) / 2f64).log(d as f64).floor() as u32;
    let (upper_inner, upper_leaf) = match h_max {
        0 => (0, 1),
        1 => (1, 2),
        _ => (
            1 + 2 * (d.pow(h_max - 2) - 1) / (d - 1),
            2 * d.pow(h_max - 2),
        ),
    };

    (
        lower_inner * inner_size + lower_leaf * leaf_size,
        upper_inner * inner_size + upper_leaf * leaf_size,
    )
}

fn btree_dynamic_usage<K, V>(entries: usize) -> usize {
    // Estimate the memory usage as the midpoint between the lower and upper bounds.
    let (lower, upper) = btree_dynamic_usage_bounds::<K, V>(entries);
    lower + ((upper - lower) / 2)
}

impl<K: DynamicUsage, V: DynamicUsage> DynamicUsage for BTreeMap<K, V> {
    fn dynamic_usage(&self) -> usize {
        btree_dynamic_usage::<K, V>(self.len())
            + self
                .iter()
                .map(|(k, v)| k.dynamic_usage() + v.dynamic_usage())
                .sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        let (lower, upper) = btree_dynamic_usage_bounds::<K, V>(self.len());
        self.iter()
            .map(|(k, v)| (k.dynamic_usage_bounds(), v.dynamic_usage_bounds()))
            .fold((lower, Some(upper)), |acc, (k, v)| {
                (
                    acc.0 + k.0 + v.0,
                    acc.1.zip(k.1).zip(v.1).map(|((a, b), c)| a + b + c),
                )
            })
    }
}

impl<T: DynamicUsage> DynamicUsage for BTreeSet<T> {
    fn dynamic_usage(&self) -> usize {
        // BTreeSet<T> is just BTreeMap<T, ()>
        btree_dynamic_usage::<T, ()>(self.len())
            + self.iter().map(DynamicUsage::dynamic_usage).sum::<usize>()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        let (lower, upper) = btree_dynamic_usage_bounds::<T, ()>(self.len());
        self.iter()
            .map(DynamicUsage::dynamic_usage_bounds)
            .fold((lower, Some(upper)), |acc, k| {
                (acc.0 + k.0, acc.1.zip(k.1).map(|(a, b)| a + b))
            })
    }
}
