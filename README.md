# bloom-rs

A basic implementation of bloom filters in rust.

# Basic Usage

```rust
extern crate bloom;
use bloom::BloomFilter;
let expected_num_items = 1000;
let false_positive_rate = 0.01;
/* make a bloomfilter for ints */
let filter:BloomFilter<int,int> = BloomFilter::with_rate(false_positive_rate,expected_num_items);
filter.insert(&1);
filter.contains(&1); /* true */
filter.contains(&2); /* false */
```

# False Positive Rate
The false positive rate is specified as a float in the range
(0,1).  If indicates that out of `X` probes, `X * rate` should
return a false positive.  Higher values will lead to smaller (but
more inaccurate) filters.

# Note on type parameters

Since at least two independent hash functions are needed for a
bloom filter, it is currently necessary to specify two types to
the the `new/with_hasher` functions.  These types *must* always be
the same.  (i.e. `BloomFilter<f32,f32>`) This is because a type
like `T: Hash<SipState> + Hash<Djb2State>` does not seem to be
currently possible. Therefore, the two requirements are split into
the two types and transmuted.
