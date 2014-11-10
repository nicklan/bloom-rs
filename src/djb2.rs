//! Basic djb2 hashing
use core::kinds::Sized;
use std::hash::{hash,Hash,Hasher,Writer};

pub struct Djb2Hasher;

pub struct Djb2State {
    res: u64,
}

impl Hasher<Djb2State> for Djb2Hasher {
    #[inline]
    fn hash<Sized? T: Hash<Djb2State>>(&self, value: &T) -> u64 {
        let mut state = Djb2State { res: 0 };
        value.hash(&mut state);
        state.res
    }
}

impl Writer for Djb2State {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        let mut hash = 5381 as u64;
        let mut i = 0;
        let lim = msg.len();
        while i < lim {
            hash = ((hash << 5) + hash) + msg[i] as u64; /* hash * 33 + c */
            i += 1
        }
        self.res = hash
    }
}
