#[allow(unused_imports)]
use builtin::*;
#[allow(unused_imports)]
use builtin_macros::*;
mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{*, vec::*, seq::*};

#[verifier(external)]
fn main() {
    crate::test::test();
}


verus! {

mod bits {
    #[allow(unused_imports)]
    use builtin::*;
    #[allow(unused_imports)]
    use builtin_macros::*;
    #[allow(unused_imports)]
    use crate::pervasive::{*, vec::*, seq::*};

    pub open spec fn bit64_get(word: u64, index: u64) -> bool
        recommends
            index < 64,
    {
        (word >> index) & 1 == 1
    }

    struct BitVector {
        bits: u64,
        len: u64,
        iterator: u64,
    }

    impl BitVector {
        pub closed spec fn inv(self) -> bool {
            true
        }

        pub closed spec fn len(self) -> int {
            self.len
        }

        pub closed spec fn iterator(self) -> int {
            self.iterator
        }

        pub closed spec fn index(self, i: int) -> bool
            recommends
                0 <= i < self.len(),
        {
            (self.bits >> i as u64) & 1 == 1
        }

        pub fn new(len: u64) -> (bv: BitVector)
            requires
                len <= 64,
            ensures
                bv.inv(),
                bv.len() == len,
                forall|i: int| 0 <= i < bv.len() ==> !bv.index(i),
        {
            let bv = BitVector { bits: 0, len, iterator: 0 };
            assert forall|i: int| 0 <= i < bv.len() implies !bv.index(i) by {
                let u = i as u64;
                assert((0u64 >> u) & 1 == 0) by(bit_vector);
            }
            bv
        }

        pub fn get(&self, n: u64) -> (b: bool)
            requires
                self.inv(),
                n < self.len(),
            ensures
                b == self.index(n),
        {
            (self.bits >> n) & 1 == 1
        }

        pub fn set(&mut self, n: u64)
            requires
                old(self).inv(),
                n < old(self).len(),
            ensures
                self.inv(),
                self.len() == old(self).len(),
                forall|i: int| 0 <= i < self.len() && i != n ==> self.index(i) == old(self).index(i),
                self.index(n),
        {
            // TODO
            assume(false);
        }

        pub fn iter_start(&mut self)
            requires
                old(self).inv(),
            ensures
                self.inv(),
                self.len() == old(self).len(),
                self.iterator() == 0,
                forall|i: int| 0 <= i < self.len() ==> self.index(i) == old(self).index(i),
        {
            self.iterator = 0;
        }

        pub fn iter_next(&mut self) {
            // TODO
        }
    }
}

mod test {
    #[allow(unused_imports)]
    use builtin::*;
    #[allow(unused_imports)]
    use builtin_macros::*;
    #[allow(unused_imports)]
    use crate::pervasive::{*, vec::*, seq::*};
    use crate::bits::*;

    pub fn test() {
    }
}

}
