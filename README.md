# bloom

An implementation of various Approximate Set Membership structures in
Rust.  Currently included are a standard Bloom Filter, and the
simplest kind of Counting Bloom Filter.

At some point more advanced types of ASMSes will be added.

# Basic Usage

```rust
extern crate bloom;
use bloom::BloomFilter;
let expected_num_items = 1000;
let false_positive_rate = 0.01;
let mut filter:BloomFilter = BloomFilter::with_rate(false_positive_rate,expected_num_items);
filter.insert(&1i);
filter.contains(&1i); /* true */
filter.contains(&2i); /* false */
```

# Installation
Use [Cargo](http://doc.crates.io/) and add the following to your Cargo.toml

```
[dependencies]
bloom="0.2.0"
```

# Documentation
See [here](https://docs.rs/bloom/)

# False Positive Rate
The false positive rate is specified as a float in the range
(0,1).  If indicates that out of `X` probes, `X * rate` should
return a false positive.  Higher values will lead to smaller (but
more inaccurate) filters.

# Benchmarks
This crate includes some benchmarks to test the performance of the
bloom filter.  To run them you'll need to use rust nightly (the
benchmark feature isn't stable yet), and then run:

```
cargo bench --features "do-bench"
```
