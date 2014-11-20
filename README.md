# bloom-rs

An implementation of bloom filters in rust.

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
[dependencies.bloom-rs]
git = "https://github.com/nicklan/bloom-rs.git"
```

# False Positive Rate
The false positive rate is specified as a float in the range
(0,1).  If indicates that out of `X` probes, `X * rate` should
return a false positive.  Higher values will lead to smaller (but
more inaccurate) filters.
