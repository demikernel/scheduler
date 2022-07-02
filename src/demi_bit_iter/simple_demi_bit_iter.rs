mod bititer {
    pub struct BitVector {
        bits: u64,
        len: u64,
        iterator: u64,
    }

    impl BitVector {
        pub fn new(len: u64) -> BitVector {
            BitVector {
                bits: 0,
                len,
                iterator: 0,
            }
        }

        pub fn get(&self, n: u64) -> bool {
            (self.bits >> n) & 1 == 1
        }

        pub fn set(&mut self, pos: u64) {
            if pos < self.len {
                self.bits = self.bits | (1 << pos);
            }
        }

        pub fn get_iter(&self) -> u64 {
            self.iterator
        }

        pub fn iter_first(&mut self) {
            let mut n: u64 = 0;
            while n < self.len && !self.get(n) {
                n = n + 1;
            }
            self.iterator = n;
        }

        // TODO: use trailing_zeros
        pub fn iter_next(&mut self) {
            let prev = self.iterator;
            let mut n: u64 = self.iterator + 1;
            while n < self.len && !self.get(n) {
                n = n + 1;
            }
            self.iterator = n;
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    // TODO: How to shorten this?
    use super::bititer::BitVector;
    #[test]
    pub fn chris_test() {
        let mut bv = BitVector::new(7);
        bv.iter_first();
        if bv.get_iter() < 7 {
            bv.iter_next();
        }
    }

    #[test]
    pub fn test_if_set_works_fine() {
        let mut bv = BitVector::new(7);
        bv.set(3);
        bv.iter_first();
        assert_eq!(bv.get_iter(), 3);
    }
}
