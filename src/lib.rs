//! Implementation of a bloom filter in rust
//
//! # Basic Usage
//!
//! ```rust,no_run
//! use bloom::BloomFilter;
//! let expected_num_items = 1000;
//! let false_positive_rate = 0.01;
//! let mut filter:BloomFilter = BloomFilter::with_rate(false_positive_rate,expected_num_items);
//! filter.insert(&1i);
//! filter.contains(&1i); /* true */
//! filter.contains(&2i); /* false */
//! ```
//!
//! # False Positive Rate
//! The false positive rate is specified as a float in the range
//! (0,1).  If indicates that out of `X` probes, `X * rate` should
//! return a false positive.  Higher values will lead to smaller (but
//! more inaccurate) filters.
//!

#![feature(default_type_params)]
#![license = "GPL2"]
extern crate collections;
extern crate core;
extern crate test;

use collections::Bitv;
use std::hash::{Hash,Hasher,RandomSipHasher};
use std::cmp::{min,max};
use std::iter::Iterator;
use std::num::Float;

pub struct BloomFilter {
    bits: Bitv,
    num_hashes: uint,
    h1: RandomSipHasher,
    h2: RandomSipHasher,
}

struct HashIter {
    h1: u64,
    h2: u64,
    i: uint,
    count: uint,
}

impl Iterator<u64> for HashIter {
    fn next(&mut self) -> Option<u64> {
        if self.i == self.count {
            return None;
        }
        let r = match self.i {
            0 => { self.h1 }
            1 => { self.h2 }
            _ => { self.h1+self.i as u64 * self.h2 }
        };
        self.i+=1;
        Some(r)
    }
}

impl BloomFilter {
    /// Create a new BloomFilter with the specified number of bits,
    /// and hashes
    pub fn with_size(num_bits: uint, num_hashes: uint) -> BloomFilter {
        BloomFilter {
            bits: Bitv::with_capacity(num_bits, false),
            num_hashes: num_hashes,
            h1: RandomSipHasher::new(),
            h2: RandomSipHasher::new(),
        }
    }

    /// create a BloomFilter that expectes to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate`.
    pub fn with_rate(rate: f32, expected_num_items: uint) -> BloomFilter {
        let bits = needed_bits(rate,expected_num_items);
        BloomFilter::with_size(bits,optimal_num_hashes(bits,expected_num_items))
    }

    /// Insert item into this bloomfilter
    pub fn insert<T: Hash>(& mut self,item: &T) {
        for h in self.get_hashes(item) {
            let idx = (h % self.bits.len() as u64) as uint;
            self.bits.set(idx,true)
        }
    }

    /// Check if the item has been inserted into this bloom filter.
    /// This function can return false positives, but no false
    /// negatives.
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for h in self.get_hashes(item) {
            let idx = (h % self.bits.len() as u64) as uint;
            if !self.bits.get(idx) {
                return false;
            }
        }
        true
    }


    fn get_hashes<T: Hash>(&self, item: &T) -> HashIter {
        let h1 = self.h1.hash(item);
        let h2 = self.h2.hash(item);
        HashIter {
            h1: h1,
            h2: h2,
            i: 0,
            count: self.num_hashes,
        }
    }
}

/// Return the optimal number of hashes to use for the given number of
/// bits and items in a filter
pub fn optimal_num_hashes(num_bits: uint, num_items: uint) -> uint {
    min(
        max(
            (num_bits as f32 / num_items as f32 * core::f32::consts::LN_2).round() as uint,
             2
           ),
        200
      )
}

/// Return the number of bits needed to satisfy the specified false
/// positive rate, if the filter will hold `num_items` items.
pub fn needed_bits(false_pos_rate:f32, num_items: uint) -> uint {
    let ln22 = core::f32::consts::LN_2 * core::f32::consts::LN_2;
    (num_items as f32 * ((1.0/false_pos_rate).ln() / ln22)).round() as uint
}


#[cfg(test)]
mod bench {
    use test::Bencher;
    use std::rand;
    use std::rand::Rng;

    use super::BloomFilter;

    #[bench]
    fn insert_benchmark(b: &mut Bencher) {
        let cnt = 500000u;
        let rate = 0.01 as f32;

        let mut bf:BloomFilter = BloomFilter::with_rate(rate,cnt);
        let mut rng = rand::task_rng();

        b.iter(|| {
            let mut i = 0;
            while i < cnt {
                let v = rng.gen::<int>();
                bf.insert(&v);
                i+=1;
            }
        })
    }

    #[bench]
    fn contains_benchmark(b: &mut Bencher) {
        let cnt = 500000u;
        let rate = 0.01 as f32;

        let mut bf:BloomFilter = BloomFilter::with_rate(rate,cnt);
        let mut rng = rand::task_rng();

        let mut i = 0;
        while i < cnt {
            let v = rng.gen::<int>();
            bf.insert(&v);
            i+=1;
        }

        b.iter(|| {
            i = 0;
            while i < cnt {
                let v = rng.gen::<int>();
                bf.contains(&v);
                i+=1;
            }
        })
    }
}

#[cfg(test)]
mod test_bloom {
    use std::collections::HashSet;
    use std::rand;
    use std::rand::Rng;

    use super::{BloomFilter,needed_bits,optimal_num_hashes};

    #[test]
    fn simple() {
        let mut b:BloomFilter = BloomFilter::with_rate(0.01,100);
        b.insert(&1i);
        assert!(b.contains(&1i));
        assert!(!b.contains(&2i));
    }

    #[test]
    fn bloom_test() {
        let cnt = 500000u;
        let rate = 0.01 as f32;

        let bits = needed_bits(rate,cnt);
        assert_eq!(bits, 4792529);
        let hashes = optimal_num_hashes(bits,cnt);
        assert_eq!(hashes, 7);

        let mut b:BloomFilter = BloomFilter::with_rate(rate,cnt);
        let mut set:HashSet<int> = HashSet::new();
        let mut rng = rand::task_rng();

        let mut i = 0;

        while i < cnt {
            let v = rng.gen::<int>();
            set.insert(v);
            b.insert(&v);
            i+=1;
        }

        i = 0;
        let mut false_positives = 0u;
        while i < cnt {
            let v = rng.gen::<int>();
            match (b.contains(&v),set.contains(&v)) {
                (true, false) => { false_positives += 1; }
                (false, true) => { assert!(false); } // should never happen
                _ => {}
            }
            i+=1;
        }

        // make sure we're not too far off
        let actual_rate = false_positives as f32 / cnt as f32;
        assert!(actual_rate > (rate-0.001));
        assert!(actual_rate < (rate+0.001));
    }
}
