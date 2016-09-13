// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation; either version 2 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA
// 02110-1301, USA.

extern crate core;
extern crate bit_vec;

use bit_vec::BitVec;

/// A ValueVec is a bit vector that holds fixed sized unsigned integer
/// values.
pub struct ValueVec {
    bits_per_val: usize,
    mask: u32,
    bits: BitVec,
}

impl ValueVec {

    /// Create a ValueVec that holds values with `bits_per_val` bits and
    /// space to hold `count` values.
    pub fn new(bits_per_val: usize, count: usize) -> ValueVec {
        let bits = bits_per_val*count;
        ValueVec {
            bits_per_val: bits_per_val,
            mask: 2u32.pow(bits_per_val as u32)-1,
            bits: BitVec::from_elem(bits,false),
        }
    }

    /// Create a ValueVec that can hold `count` values where the
    /// maximam value of each entry is at least `max_val` (inclusive)
    ///
    /// # Example
    ///
    /// ```rust,should_panic
    /// use bloom::ValueVec;
    /// let mut vv = ValueVec::with_max(7,3);
    /// vv.set(0,7); // okay
    /// vv.set(0,8); // will panic
    /// ```
    pub fn with_max(max_val: u32, count: usize) -> ValueVec {
        let mut bits_per_val = 0;
        let mut cur = max_val;
        // there are fancy faster versions of this, but this is only
        // run in a constructor, so no need to complicate things
        while cur > 0 {
            bits_per_val+=1;
            cur>>=1;
        }
        ValueVec::new(bits_per_val,count)
    }

    /// How many bits this ValueVec is using to store each value
    pub fn bits_per_val(&self) -> usize {
        self.bits_per_val
    }

    /// The maximum value this ValueVec can hold per entry
    pub fn max_value(&self) -> u32 {
        self.mask
    }

    /// Resets all values to 0 in this ValueVec
    pub fn clear(&mut self) {
        self.bits.clear();
    }

    fn set_bits(&mut self, idx: usize,  val: u32, num_bits: usize) {
        let mut blocks = unsafe {self.bits.storage_mut()};
        let blockidx = idx/32;
        let shift = 32-(idx%32)-num_bits;
        let mask =
            if num_bits==self.bits_per_val {
                self.mask
            } else {
                2u32.pow(num_bits as u32)-1
            } << shift;
        let block = blocks[blockidx];

        // this will be the value with all bits in our value set to zero
        let zeroed = (block ^ mask) & block;
        // or in the new val
        blocks[blockidx] = zeroed | (val<<shift);
    }

    fn get_bits(&self, idx: usize, num_bits: usize) -> u32 {
        let blocks = self.bits.storage();
        let shift = 32-(idx%32)-num_bits;
        let mask =
            if num_bits==self.bits_per_val {
                self.mask
            } else {
                2u32.pow(num_bits as u32)-1
            } << shift;
        let val = blocks[idx/32] & mask;
        val >> shift
    }

    /// Get the total number of bits this valuevec is using
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Set value at index `i` to value `val`.
    ///
    /// # Panics
    ///
    /// Panics if `val` needs more bits to store than the number of
    /// bits this vec is using per value
    pub fn set(&mut self, i: usize, val: u32) {
        if val > self.mask {
            panic!("set with val {}, max value this ValueVec can hold is {}",
                   val,self.mask);
        }
        let idx = i*self.bits_per_val;
        //println!("idx is: {}",idx);
        let rem = 32-(idx%32);
        if rem < self.bits_per_val {
            // rem is how many bits needed in the lower part
            let left = self.bits_per_val-rem;
            let lowerval = val>>left;
            self.set_bits(idx,lowerval,rem);

            // now put the rest of the bits in
            let upval = val&(2u32.pow(left as u32)-1);
            self.set_bits(idx+rem,upval,left);
        } else {
            let vs = self.bits_per_val;
            self.set_bits(idx,val,vs);
        }
    }

    /// Get the value in this ValueVec stored at index `i`
    pub fn get(&self, i: usize) -> u32 {
        let idx = i*self.bits_per_val;
        let rem = 32-(idx%32);
        if rem < self.bits_per_val {
            let lower = self.get_bits(idx,rem);
            let left = self.bits_per_val-rem;
            let upper = self.get_bits(idx+rem,left);
            (lower<<left)|upper
        } else {
            self.get_bits(idx,self.bits_per_val)
        }
    }
}

#[cfg(test)]
mod tests {
    use valuevec::ValueVec;

    #[test]
    fn set_get_no_overlap() {
        let mut vv = ValueVec::new(4,12);

        vv.set(1,3);
        assert_eq!(vv.get(1),3);

        vv.set(2,4);
        assert_eq!(vv.get(1),3);
        assert_eq!(vv.get(2),4);

        vv.set(11,2);
        assert_eq!(vv.get(1),3);
        assert_eq!(vv.get(2),4);
        assert_eq!(vv.get(11),2);
    }

    #[test]
    fn set_get_overlap() {
        let mut vv = ValueVec::new(3,12);

        vv.set(1,3);
        assert_eq!(vv.get(1),3);

        vv.set(2,4);
        assert_eq!(vv.get(1),3);
        assert_eq!(vv.get(2),4);

        vv.set(10,7);
        assert_eq!(vv.get(1),3);
        assert_eq!(vv.get(2),4);
        assert_eq!(vv.get(10),7);

        vv.set(11,2);
        assert_eq!(vv.get(1),3);
        assert_eq!(vv.get(2),4);
        assert_eq!(vv.get(10),7);
        assert_eq!(vv.get(11),2);
    }

    #[test]
    #[should_panic]
    fn set_over_max() {
        let mut vv = ValueVec::new(2,2);
        vv.set(0,100);
    }

    #[test]
    fn with_max() {
        let mut vv = ValueVec::with_max(35,3);
        vv.set(0,35);
        assert_eq!(vv.get(0),35);
        vv.set(2,14);
        assert_eq!(vv.get(0),35);
        assert_eq!(vv.get(2),14);
        assert_eq!(vv.get(1),0);
    }

    #[test]
    #[should_panic]
    fn over_with_max() {
        let mut vv = ValueVec::with_max(7,2);
        vv.set(0,7);
        vv.set(1,8);
    }
}
