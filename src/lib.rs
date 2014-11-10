//! Implementation of a bloom filter in rust
//
//! # Basic Usage
//!
//! ```
//! extern crate bloom;
//! use bloom::BloomFilter;
//! let expected_num_items = 1000;
//! let false_positive_rate = 0.01;
//! /* make a bloomfilter for ints */
//! let filter:BloomFilter<int,int> = BloomFilter::with_rate(false_positive_rate,expected_num_items);
//! filter.insert(1);
//! filter.contains(1); /* true */
//! filter.contains(2); /* false */
//! ```
//!
//! # False Positive Rate
//! The false positive rate is specified as a float in the range
//! (0,1).  If indicates that out of `X` probes, `X * rate` should
//! return a false positive.  Higher values will lead to smaller (but
//! more inaccurate) filters.
//!
//! # Note on type parameters
//!
//! Since at least two independent hash functions are needed for a
//! bloom filter, it is currently necessary to specify two types to
//! the the `new/with_hasher` functions.  These types *must* always be
//! the same.  (i.e. `BloomFilter<f32,f32>`) This is because a type
//! like `T: Hash<SipState> + Hash<Djb2State>` does not seem to be
//! currently possible. Therefore, the two requirements are split into
//! the two types and transmuted.

#![feature(default_type_params)]
extern crate collections;
extern crate core;

use collections::Bitv;
use std::hash::{hash,Hash,Hasher};
use std::cmp::{min,max};

use djb2::{Djb2Hasher,Djb2State};
mod djb2;


pub struct BloomFilter<T, U, H = Djb2Hasher> {
    bits: Bitv,
    num_hashes: uint,
    hasher: H,
}

impl<T: Hash, U: Hash<Djb2State>> BloomFilter<T,U,Djb2Hasher> {
    /// create a BloomFilter that expectes to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate` and
    /// will use a Djb2 hasher.
    pub fn with_rate(rate: f32, expected_num_items: uint) -> BloomFilter<T,U,Djb2Hasher> {
        let bits = needed_bits(rate,expected_num_items);
        BloomFilter::new(bits,optimal_num_hashes(bits,expected_num_items))
    }

    /// Create a new BloomFilter with the specified number of bits,
    /// hashes, and a Djb2 hasher.
    pub fn new(num_bits: uint, num_hashes: uint) -> BloomFilter<T,U,Djb2Hasher> {
        BloomFilter::with_hasher(num_bits,num_hashes,Djb2Hasher)
    }
}

impl<T: Hash, U: Hash<S>, S, H:Hasher<S>> BloomFilter<T,U,H> {
    /// create a BloomFilter that expectes to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate` and
    /// will use the specified hasher
    pub fn with_rate_and_hasher(rate: f32, expected_num_items: uint, hasher: H) -> BloomFilter<T,U,H> {
        let bits = needed_bits(rate,expected_num_items);
        BloomFilter::with_hasher(bits,optimal_num_hashes(bits,expected_num_items),hasher)
    }

    /// Create a BloomFilter with the specified number of bits,
    /// hashes, and the specified hasher.
    pub fn with_hasher(num_bits: uint, num_hashes: uint, hasher: H) -> BloomFilter<T,U,H> {
        BloomFilter {
            bits: Bitv::with_capacity(num_bits, false),
            num_hashes: num_hashes,
            hasher: hasher,
        }
    }

    /// Insert item into this bloomfilter
    pub fn insert(& mut self,item: T) {
        for h in self.get_hashes(&item).iter() {
            let idx = (h % self.bits.len() as u64) as uint;
            self.bits.set(idx,true)
        }
    }

    /// Check if the item has been inserted into this bloom filter.
    /// This function can return false positives, but no false
    /// negatives.
    pub fn contains(&self, item: &T) -> bool {
        for h in self.get_hashes(item).iter() {
            let idx = (h % self.bits.len() as u64) as uint;
            if !self.bits.get(idx) {
                return false;
            }
        }
        true
    }


    fn get_hashes(&self, item: &T) -> Vec<u64> {
        let mut vec = Vec::with_capacity(self.num_hashes);
        let h1 = hash(item);
        let h2 =
            unsafe {
                let u:&U = std::mem::transmute(item);
                self.hasher.hash(u)
            };
        for i in range(0,self.num_hashes) {
            vec.push((h1+i as u64 *h2))
        }
        vec
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
mod test_bloom {
    use std::collections::HashSet;
    use std::rand;
    use std::rand::Rng;

    use super::{BloomFilter,needed_bits,optimal_num_hashes};

    #[test]
    fn bloom_test() {
        let cnt = 500000u;
        let rate = 0.01 as f32;

        let bits = needed_bits(rate,cnt);
        assert_eq!(bits, 4792529);
        let hashes = optimal_num_hashes(bits,cnt);
        assert_eq!(hashes, 7);

        let mut b:BloomFilter<int,int> = BloomFilter::new(bits,hashes);
        let mut set:HashSet<int> = HashSet::new();
        let mut rng = rand::task_rng();

        let mut i = 0;

        while i < cnt {
            let v = rng.gen::<int>();
            set.insert(v);
            b.insert(v);
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


