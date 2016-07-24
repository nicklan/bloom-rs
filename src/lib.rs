//! Implementation of a bloom filter in rust
//! # Basic Usage
//! ```toml
//! [dependencies]
//! bloom = "0.2.0"
//! ```
//!
//! add this to your crate root:
//!
//! ```rust
//! extern crate bloom;
//! ```
//!

#![crate_name="bloom"]
#![crate_type = "rlib"]

#![cfg_attr(feature = "do-bench", feature(test))]

extern crate core;
extern crate bit_vec;

pub mod bloom;
pub use bloom::{BloomFilter,optimal_num_hashes,needed_bits};

/// Filters that implement this trait can be intersected with filters
/// of the same type to produce a filter that contains the
/// items that have been inserted into *both* filters.
///
/// Both filters MUST be the same size and be using the same hash
/// functions for this to work.  Will panic if the filters are not the
/// same size, but will simply produce incorrect (meaningless) results
/// if the filters are using different hash functions.
pub trait Intersectable {
    fn intersect(&mut self, other: &Self) -> bool;
}

/// Filters that implement this trait can be unioned with filters
/// of the same type to produce a filter that contains the
/// items that have been inserted into *either* filter.
///
/// Both filters MUST be the same size and be using the same hash
/// functions for this to work.  Will panic if the filters are not the
/// same size, but will simply produce incorrect (meaningless) results
/// if the filters are using different hash functions.
pub trait Unionable {
    fn union(&mut self, other: &Self) -> bool;
}

/// Filters than are Combineable can be unioned and intersected
pub trait Combineable: Intersectable + Unionable {}
impl<T> Combineable for T where T: Intersectable + Unionable {}
