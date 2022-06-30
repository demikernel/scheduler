#[allow(unused_imports)]
use builtin::*;
#[allow(unused_imports)]
use builtin_macros::*;
mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{*, vec::*, seq::*};

#[verifier(external)]
fn main() {
    crate::trusted::test();
}


verus! {

mod trusted {
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

    #[verifier(external_body)]
    pub struct State {
        word: std::marker::PhantomData<u64>,
        index: std::marker::PhantomData<u64>,
    }

    impl State {
        pub spec fn word(self) -> u64;
        pub spec fn index(self) -> u64;
    }

    #[verifier(external_body)]
    pub fn skip_bits(n: u64, state: State) -> (next: State)
        requires
            state.index() + n <= 64,
            forall|i: u64| state.index() <= i && i < state.index() + n ==> !bit64_get(state.word(), i),
        ensures
            next.index() == state.index() + n,
            next.word() == state.word(),
    {
        State { word: Default::default(), index: Default::default() }
    }

    #[verifier(external_body)]
    pub fn execute_bit(index: u64, state: State) -> (next: State)
        requires
            index == state.index(),
            index < 64,
            bit64_get(state.word(), index),
        ensures
            next.index() == state.index() + 1,
            next.word() == state.word(),
    {
        println!("executing bit {index}");
        State { word: Default::default(), index: Default::default() }
    }

    #[verifier(external)]
    pub fn test() {
        println!("test1");
        let state = State { word: Default::default(), index: Default::default() };
        crate::implementation::execute_bits(5, state);

        println!("test2");
        let state = State { word: Default::default(), index: Default::default() };
        crate::implementation::execute_bits(24, state);
    }
}

mod implementation {
    #[allow(unused_imports)]
    use builtin::*;
    #[allow(unused_imports)]
    use builtin_macros::*;
    #[allow(unused_imports)]
    use crate::pervasive::{*, vec::*, seq::*};
    use crate::trusted::*;

    pub fn execute_bits(word: u64, state: State) -> (last: State)
        requires
            state.word() == word,
            state.index() == 0,
        ensures
            last.index() == 3,
    {
        // bits 0, 1, 2
        if word & 7 == 0 {
            assert(word & 7 == 0 ==> (word >> 0u64) & 1 == 0) by(bit_vector);
            assert(word & 7 == 0 ==> (word >> 1u64) & 1 == 0) by(bit_vector);
            assert(word & 7 == 0 ==> (word >> 2u64) & 1 == 0) by(bit_vector);
            return skip_bits(3, state);
        }

        // bit 0
        assert(word >> 0u64 == word) by(bit_vector);
        let state = if word & 1 == 1 {
            execute_bit(0, state)
        } else {
            skip_bits(1, state)
        };

        // bit 1
        let state = if (word >> 1) & 1 == 1 {
            execute_bit(1, state)
        } else {
            skip_bits(1, state)
        };

        // bit 2
        let state = if (word >> 2) & 1 == 1 {
            execute_bit(2, state)
        } else {
            skip_bits(1, state)
        };

        state
    }
}

}
