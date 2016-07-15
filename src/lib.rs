//! Implementation of a bloom filter in rust
//
//! # Basic Usage
//!
//! ```rust,no_run
//! use bloom::BloomFilter;
//! let expected_num_items = 1000;
//! let false_positive_rate = 0.01;
//! let mut filter:BloomFilter = BloomFilter::with_rate(false_positive_rate,expected_num_items);
//! filter.insert(&1);
//! filter.contains(&1); /* true */
//! filter.contains(&2); /* false */
//! ```
//!
//! # False Positive Rate
//! The false positive rate is specified as a float in the range
//! (0,1).  If indicates that out of `X` probes, `X * rate` should
//! return a false positive.  Higher values will lead to smaller (but
//! more inaccurate) filters.
//!

#![cfg_attr(feature = "do-bench", feature(test))]

extern crate core;
extern crate bit_vec;
extern crate rand;

use bit_vec::BitVec;
use rand::Rng;
use std::cmp::{min,max};
use std::hash::{Hash,Hasher,SipHasher};
use std::iter::Iterator;

pub struct BloomFilter {
    bits: BitVec,
    num_hashes: u32,
    h1: SipHasher,
    h2: SipHasher,
}

struct HashIter {
    h1: u64,
    h2: u64,
    i: u32,
    count: u32,
}

impl Iterator for HashIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.i == self.count {
            return None;
        }
        let r = match self.i {
            0 => { self.h1 }
            1 => { self.h2 }
            _ => {
                let p1 = self.h1.wrapping_add(self.i as u64);
                p1.wrapping_mul(self.h2)
            }
        };
        self.i+=1;
        Some(r)
    }
}

impl BloomFilter {
    /// Create a new BloomFilter with the specified number of bits,
    /// and hashes
    pub fn with_size(num_bits: usize, num_hashes: u32) -> BloomFilter {
        let mut rng = rand::thread_rng();
        BloomFilter {
            bits: BitVec::from_elem(num_bits,false),
            num_hashes: num_hashes,
            h1: SipHasher::new_with_keys(rng.gen::<u64>(),rng.gen::<u64>()),
            h2: SipHasher::new_with_keys(rng.gen::<u64>(),rng.gen::<u64>()),
        }
    }

    /// create a BloomFilter that expectes to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate`.
    pub fn with_rate(rate: f32, expected_num_items: u32) -> BloomFilter {
        let bits = needed_bits(rate,expected_num_items);
        BloomFilter::with_size(bits,optimal_num_hashes(bits,expected_num_items))
    }

    /// Get the number of bits this BloomFilter is using
    pub fn num_bits(&self) -> usize {
        self.bits.len()
    }

    /// Get the number of hash functions this BloomFilter is using
    pub fn num_hashes(&self) -> u32 {
        self.num_hashes
    }

    /// Insert item into this bloomfilter
    pub fn insert<T: Hash>(& mut self,item: &T) {
        for h in self.get_hashes(item) {
            let idx = (h % self.bits.len() as u64) as usize;
            self.bits.set(idx,true)
        }
    }

    /// Check if the item has been inserted into this bloom filter.
    /// This function can return false positives, but not false
    /// negatives.
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for h in self.get_hashes(item) {
            let idx = (h % self.bits.len() as u64) as usize;
            match self.bits.get(idx) {
                Some(b) => {
                    if !b {
                        return false;
                    }
                }
                None => { panic!("Hash mod failed"); }
            }
        }
        true
    }

    /// Remove all values from this BloomFilter
    pub fn clear(&mut self) {
        self.bits.clear();
    }


    fn get_hashes<T: Hash>(&self, item: &T) -> HashIter {
        let mut h1 = self.h1.clone();
        let mut h2 = self.h2.clone();
        item.hash(&mut h1);
        item.hash(&mut h2);
        let h1 = h1.finish();
        let h2 = h2.finish();
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
pub fn optimal_num_hashes(num_bits: usize, num_items: u32) -> u32 {
    min(
        max(
            (num_bits as f32 / num_items as f32 * core::f32::consts::LN_2).round() as u32,
             2
           ),
        200
      )
}

/// Return the number of bits needed to satisfy the specified false
/// positive rate, if the filter will hold `num_items` items.
pub fn needed_bits(false_pos_rate:f32, num_items: u32) -> usize {
    let ln22 = core::f32::consts::LN_2 * core::f32::consts::LN_2;
    (num_items as f32 * ((1.0/false_pos_rate).ln() / ln22)).round() as usize
}

#[cfg(feature = "do-bench")]
#[cfg(test)]
mod bench {
    extern crate test;
    use self::test::Bencher;
    use rand::{self,Rng};

    use super::BloomFilter;

    #[bench]
    fn insert_benchmark(b: &mut Bencher) {
        let cnt = 500000;
        let rate = 0.01 as f32;

        let mut bf:BloomFilter = BloomFilter::with_rate(rate,cnt);
        let mut rng = rand::thread_rng();

        b.iter(|| {
            let mut i = 0;
            while i < cnt {
                let v = rng.gen::<i32>();
                bf.insert(&v);
                i+=1;
            }
        })
    }

    #[bench]
    fn contains_benchmark(b: &mut Bencher) {
        let cnt = 500000;
        let rate = 0.01 as f32;

        let mut bf:BloomFilter = BloomFilter::with_rate(rate,cnt);
        let mut rng = rand::thread_rng();

        let mut i = 0;
        while i < cnt {
            let v = rng.gen::<i32>();
            bf.insert(&v);
            i+=1;
        }

        b.iter(|| {
            i = 0;
            while i < cnt {
                let v = rng.gen::<i32>();
                bf.contains(&v);
                i+=1;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use rand::{self,Rng};
    use super::{BloomFilter,needed_bits,optimal_num_hashes};

    #[test]
    fn simple() {
        let mut b:BloomFilter = BloomFilter::with_rate(0.01,100);
        b.insert(&1);
        assert!(b.contains(&1));
        assert!(!b.contains(&2));
        b.clear();
        assert!(!b.contains(&1));
    }

    #[test]
    fn bloom_test() {
        let cnt = 500000;
        let rate = 0.01 as f32;

        let bits = needed_bits(rate,cnt);
        assert_eq!(bits, 4792529);
        let hashes = optimal_num_hashes(bits,cnt);
        assert_eq!(hashes, 7);

        let mut b:BloomFilter = BloomFilter::with_rate(rate,cnt);
        let mut set:HashSet<i32> = HashSet::new();
        let mut rng = rand::thread_rng();

        let mut i = 0;

        while i < cnt {
            let v = rng.gen::<i32>();
            set.insert(v);
            b.insert(&v);
            i+=1;
        }

        i = 0;
        let mut false_positives = 0;
        while i < cnt {
            let v = rng.gen::<i32>();
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
