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

#![crate_name="bloom"]
#![crate_type = "rlib"]

#![cfg_attr(feature = "do-bench", feature(test))]

extern crate core;
extern crate bit_vec;

use bit_vec::BitVec;
use std::cmp::{min,max};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher,Hash,Hasher};
use std::iter::Iterator;

pub struct BloomFilter<R = RandomState, S = RandomState> {
    bits: BitVec,
    num_hashes: u32,
    hash_builder_one: R,
    hash_builder_two: S,
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

impl BloomFilter<RandomState, RandomState> {
    /// Create a new BloomFilter with the specified number of bits,
    /// and hashes
    pub fn with_size(num_bits: usize, num_hashes: u32) -> BloomFilter<RandomState, RandomState> {
        BloomFilter {
            bits: BitVec::from_elem(num_bits,false),
            num_hashes: num_hashes,
            hash_builder_one: RandomState::new(),
            hash_builder_two: RandomState::new(),
        }
    }

    /// create a BloomFilter that expects to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate`.
    pub fn with_rate(rate: f32, expected_num_items: u32) -> BloomFilter<RandomState, RandomState> {
        let bits = needed_bits(rate,expected_num_items);
        BloomFilter::with_size(bits,optimal_num_hashes(bits,expected_num_items))
    }
}

impl<R,S> BloomFilter<R,S>
    where R: BuildHasher, S: BuildHasher
{

    /// Create a new BloomFilter with the specified number of bits,
    /// hashes, and the two specified HashBuilders.  Note the the
    /// HashBuilders MUST provide independent hash values.  Passing
    /// two HashBuilders that produce the same or correlated hash
    /// values will break the false positive guarantees of the
    /// BloomFilter.
    pub fn with_size_and_hashers(num_bits: usize, num_hashes: u32,
                                 hash_builder_one: R, hash_builder_two: S) -> BloomFilter<R,S> {
        BloomFilter {
            bits: BitVec::from_elem(num_bits,false),
            num_hashes: num_hashes,
            hash_builder_one: hash_builder_one,
            hash_builder_two: hash_builder_two,
        }
    }

    /// create a BloomFilter that expects to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate`.  Items
    /// will be hashed using the Hashers produced by
    /// `hash_builder_one` and `hash_builder_two`.  Note the the
    /// HashBuilders MUST provide independent hash values.  Passing
    /// two HashBuilders that produce the same or correlated hash
    /// values will break the false positive guarantees of the
    /// BloomFilter.
    pub fn with_rate_and_hashers(rate: f32, expected_num_items: u32,
                                 hash_builder_one: R, hash_builder_two: S) -> BloomFilter<R, S> {
        let bits = needed_bits(rate,expected_num_items);
        BloomFilter::with_size_and_hashers(bits,optimal_num_hashes(bits,expected_num_items),
                                           hash_builder_one,hash_builder_two)
    }

    /// Get the number of bits this BloomFilter is using
    pub fn num_bits(&self) -> usize {
        self.bits.len()
    }

    /// Get the number of hash functions this BloomFilter is using
    pub fn num_hashes(&self) -> u32 {
        self.num_hashes
    }

    /// Insert item into this BloomFilter.
    ///
    /// If the BloomFilter did not have this value present, `true` is returned.
    ///
    /// If the BloomFilter did have this value present, `false` is returned.
    pub fn insert<T: Hash>(& mut self,item: &T) -> bool {
        let mut contained = true;
        for h in self.get_hashes(item) {
            let idx = (h % self.bits.len() as u64) as usize;
            match self.bits.get(idx) {
                Some(b) => {
                    if !b {
                        contained = false;
                    }
                }
                None => { panic!("Hash mod failed in insert"); }
            }
            self.bits.set(idx,true)
        }
        !contained
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
        let mut b1 = self.hash_builder_one.build_hasher();
        let mut b2 = self.hash_builder_two.build_hasher();
        item.hash(&mut b1);
        item.hash(&mut b2);
        let h1 = b1.finish();
        let h2 = b2.finish();
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

#[cfg(test)]
extern crate rand;

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
