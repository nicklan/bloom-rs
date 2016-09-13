
use std::hash::{BuildHasher,Hash};
use std::collections::hash_map::RandomState;
use super::ValueVec;
use super::ASMS;
use super::hashing::HashIter;

/// A standard counting bloom filter that uses a fixed number of bits
/// per counter, supports remove, and estimating the count of the
/// number of items inserted.
pub struct CountingBloomFilter<R = RandomState, S = RandomState> {
    counters: ValueVec,
    num_entries: u64,
    num_hashes: u32,
    hash_builder_one: R,
    hash_builder_two: S,
}


impl CountingBloomFilter<RandomState,RandomState> {
    /// Create a new CountingBloomFilter that will hold `num_entries`
    /// items, uses `bits_per_entry` per item, and `num_hashes` hashes
    pub fn with_size(num_entries: usize,
                     bits_per_entry: usize,
                     num_hashes: u32) -> CountingBloomFilter<RandomState,RandomState> {
        CountingBloomFilter {
            counters: ValueVec::new(bits_per_entry, num_entries),
            num_entries: num_entries as u64,
            num_hashes: num_hashes,
            hash_builder_one: RandomState::new(),
            hash_builder_two: RandomState::new(),
        }
    }

    /// create a CountingBloomFilter that uses `bits_per_entry`
    /// entries and expects to hold `expected_num_items`.  The filter
    /// will be sized to have a false positive rate of the value
    /// specified in `rate`.
    pub fn with_rate(bits_per_entry: usize, rate: f32, expected_num_items: u32) -> CountingBloomFilter<RandomState, RandomState> {
        let entries = super::bloom::needed_bits(rate,expected_num_items);
        CountingBloomFilter::with_size(entries,
                                       bits_per_entry,
                                       super::bloom::optimal_num_hashes(entries,expected_num_items))
    }

    /// Return the number of bits needed to hold values up to and
    /// including `max`
    ///
    /// # Example
    ///
    /// ```rust
    /// use bloom::CountingBloomFilter;
    /// // Create a CountingBloomFilter that can count up to 10 on each entry, and with 1000
    /// // items will have a false positive rate of 0.01
    /// let cfb = CountingBloomFilter::with_rate(CountingBloomFilter::bits_for_max(10),
    ///                                          0.01,
    ///                                          1000);
    /// ```
    pub fn bits_for_max(max: u32) -> usize {
        let mut bits_per_val = 0;
        let mut cur = max;
        while cur > 0 {
            bits_per_val+=1;
            cur>>=1;
        }
        bits_per_val
    }
}

impl<R,S> CountingBloomFilter<R,S>
    where R: BuildHasher, S: BuildHasher
{
    /// Create a new CountingBloomFilter with the specified number of
    /// bits, hashes, and the two specified HashBuilders.  Note the
    /// the HashBuilders MUST provide independent hash values.
    /// Passing two HashBuilders that produce the same or correlated
    /// hash values will break the false positive guarantees of the
    /// CountingBloomFilter.
    pub fn with_size_and_hashers(num_entries: usize,
                                 bits_per_entry: usize,
                                 num_hashes: u32,
                                 hash_builder_one: R, hash_builder_two: S) -> CountingBloomFilter<R,S> {
        CountingBloomFilter {
            counters: ValueVec::new(bits_per_entry, num_entries),
            num_entries: num_entries as u64,
            num_hashes: num_hashes,
            hash_builder_one: hash_builder_one,
            hash_builder_two: hash_builder_two,
        }
    }

    /// Create a CountingBloomFilter that expects to hold
    /// `expected_num_items`.  The filter will be sized to have a
    /// false positive rate of the value specified in `rate`.  Items
    /// will be hashed using the Hashers produced by
    /// `hash_builder_one` and `hash_builder_two`.  Note the the
    /// HashBuilders MUST provide independent hash values.  Passing
    /// two HashBuilders that produce the same or correlated hash
    /// values will break the false positive guarantees of the
    /// CountingBloomFilter.
    pub fn with_rate_and_hashers(bits_per_entry: usize, rate: f32, expected_num_items: u32,
                                 hash_builder_one: R, hash_builder_two: S) -> CountingBloomFilter<R, S> {
        let entries = super::bloom::needed_bits(rate,expected_num_items);
        CountingBloomFilter::with_size_and_hashers(entries,bits_per_entry,
                                                   super::bloom::optimal_num_hashes(entries,expected_num_items),
                                                   hash_builder_one,hash_builder_two)
    }

    /// Remove an item.  Returns an upper bound of the number of times
    /// this item had been inserted previously (i.e. the count before
    /// this remove).  Returns 0 if item was never inserted.
    pub fn remove<T: Hash>(&mut self, item: &T) ->  u32 {
        if !(self as &CountingBloomFilter<R,S>).contains(item) {
            return 0;
        }
        let mut min = u32::max_value();
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur < min {
                min = cur;
            }
            if cur > 0 {
                self.counters.set(idx,cur-1);
            } else {
                panic!("Contains returned true but a counter is 0");
            }
        }
        min
    }

    /// Return an estimate of the number of times `item` has been
    /// inserted into the filter.  Esitimate is a upper bound on the
    /// count, meaning the item has been inserted *at most* this many
    /// times, but possibly fewer.
    pub fn estimate_count<T: Hash>(&self, item: &T) -> u32 {
        let mut min = u32::max_value();
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur < min {
                min = cur;
            }
        }
        min
    }

    /// Inserts an item, returns the estimated count of the number of
    /// times this item had previously been inserted (not counting
    /// this insertion)
    pub fn insert_get_count<T: Hash>(&mut self, item: &T) -> u32 {
        let mut min = u32::max_value();
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur < min {
                min = cur;
            }
            if cur < self.counters.max_value() {
                self.counters.set(idx,cur+1);
            }
        }
        min
    }
}

impl<R,S> ASMS for CountingBloomFilter<R,S>
    where R: BuildHasher, S: BuildHasher {
    /// Inserts an item, returns true if this item was already in the
    /// filter any number of times
    fn insert<T: Hash>(&mut self, item: &T) -> bool {
        let mut min = u32::max_value();
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur < min {
                min = cur;
            }
            if cur < self.counters.max_value() {
                self.counters.set(idx,cur+1);
            }
        }
        min > 0
    }


    /// Check if the item has been inserted into this
    /// CountingBloomFilter.  This function can return false
    /// positives, but not false negatives.
    fn contains<T: Hash>(&self, item: &T) -> bool {
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur == 0 {
                return false;
            }
        }
        true
    }

    /// Remove all values from this CountingBloomFilter
    fn clear(&mut self) {
        self.counters.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::CountingBloomFilter;
    use ASMS;

    #[test]
    fn simple() {
        let mut cbf:CountingBloomFilter = CountingBloomFilter::with_rate(4,0.01,100);
        assert_eq!(cbf.insert(&1),false);
        assert!(cbf.contains(&1));
        assert!(!cbf.contains(&2));
    }

    #[test]
    fn remove() {
        let mut cbf:CountingBloomFilter = CountingBloomFilter::with_rate(CountingBloomFilter::bits_for_max(10)
                                                                         ,0.01,100);
        assert_eq!(cbf.insert_get_count(&1),0);
        cbf.insert(&2);
        assert!(cbf.contains(&1));
        assert!(cbf.contains(&2));
        assert_eq!(cbf.remove(&2),1);
        assert_eq!(cbf.remove(&3),0);
        assert!(cbf.contains(&1));
        assert!(!cbf.contains(&2));
    }

    #[test]
    fn estimate_count() {
        let mut cbf:CountingBloomFilter = CountingBloomFilter::with_rate(4,0.01,100);
        cbf.insert(&1);
        cbf.insert(&2);
        assert_eq!(cbf.estimate_count(&1),1);
        assert_eq!(cbf.estimate_count(&2),1);
        assert_eq!(cbf.insert_get_count(&1),1);
        assert_eq!(cbf.estimate_count(&1),2);
    }
}

