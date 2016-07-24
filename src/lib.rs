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
pub use bloom::BloomFilter;
