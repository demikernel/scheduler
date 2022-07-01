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

pub open spec fn bit64_get(word: u64, index: u64) -> bool
    recommends
        index < 64,
{
    (word >> index) & 1 == 1
}

#[verifier(external_body)]
fn trailing_zeros(u: u64) -> (n: u64)
    ensures
        n <= 64,
        n < 64 ==> bit64_get(u, n),
        forall|i: u64| i < n ==> !bit64_get(u, i),
{
    u.trailing_zeros() as u64
}

mod bits {
    #[allow(unused_imports)]
    use builtin::*;
    #[allow(unused_imports)]
    use builtin_macros::*;
    #[allow(unused_imports)]
    use crate::pervasive::{*, vec::*, seq::*, modes::*};
    #[allow(unused_imports)]
    use crate::bit64_get;

    pub struct BitVector {
        bits: u64,
        len: u64,
        iterator: u64,
    }

    impl BitVector {
        pub closed spec fn inv(self) -> bool {
            self.len <= 64
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
            bit64_get(self.bits, i as u64)
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

        pub fn get_iter(&self) -> (n: u64)
            ensures
                n == self.iterator(),
        {
            self.iterator
        }

        // TODO: use trailing_zeros
        pub fn iter_first(&mut self)
            requires
                old(self).inv(),
            ensures
                self.inv(),
                self.len() == old(self).len(),
                self.iterator() <= self.len(),
                self.iterator() < self.len() ==> self.index(self.iterator()),
                forall|i: int| 0 <= i < self.iterator() ==> !self.index(i),
                forall|i: int| 0 <= i < self.len() ==> self.index(i) == old(self).index(i),
        {
            let mut n: u64 = 0;
            while n < self.len && !self.get(n)
                invariant
                    self.inv(),
                    forall|i: u64| i < n ==> !bit64_get(self.bits, i),
                    n <= self.len(),
            {
                n = n + 1;
            }
            self.iterator = n;
        }

        // TODO: use trailing_zeros
        pub fn iter_next(&mut self)
            requires
                old(self).inv(),
                old(self).iterator() < old(self).len(),
            ensures
                self.inv(),
                self.len() == old(self).len(),
                old(self).iterator() < self.iterator() <= self.len(),
                self.iterator() < self.len() ==> self.index(self.iterator()),
                forall|i: int| old(self).iterator() < i < self.iterator() ==> !self.index(i),
                forall|i: int| 0 <= i < self.len() ==> self.index(i) == old(self).index(i),
        {
            let prev = self.iterator;
            let mut n: u64 = self.iterator + 1;
            while n < self.len && !self.get(n)
                invariant
                    self.inv(),
                    forall|i: u64| prev < i < n ==> !bit64_get(self.bits, i),
                    prev < n <= self.len(),
            {
                n = n + 1;
            }
            self.iterator = n;
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
    #[allow(unused_imports)]
    use crate::bits::*;

    pub fn test() {
        let mut bv = BitVector::new(7);
        bv.iter_first();
        if bv.get_iter() < 7 {
            bv.iter_next();
        }
    }
}

}
