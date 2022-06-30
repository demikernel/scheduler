#[allow(unused_imports)]
use builtin::*;
#[allow(unused_imports)]
use builtin_macros::*;
mod pervasive;
#[allow(unused_imports)]
use crate::pervasive::{*, vec::*, seq::*};

#[verifier(external)]
fn main() {}


verus! {

mod trusted {
    #[allow(unused_imports)]
    use builtin::*;
    #[allow(unused_imports)]
    use builtin_macros::*;
    #[allow(unused_imports)]
    use crate::pervasive::{*, vec::*, seq::*, modes::*};

    pub open spec fn bit64_get(word: u64, index: u64) -> bool
        recommends
            index < 64,
    {
        (word >> index) & 1 == 1
    }

    pub struct State {
        word: u64,
        index: u64,
    }

    impl State {
        pub closed spec fn word(self) -> u64 {
            self.word
        }

        pub closed spec fn index(self) -> u64 {
            self.index
        }
    }

    #[verifier(external_body)]
    pub proof fn skip_bit(tracked state: State) -> (tracked next: State)
        requires
            state.index() < 64,
            !bit64_get(state.word(), state.index()),
        ensures
            next.index() == state.index() + 1,
            next.word() == state.word(),
    {
        unimplemented!()
    }

    #[verifier(external_body)]
    pub fn execute_bit(index: u64, state: Tracked<State>) -> (next: Tracked<State>)
        requires
            index == (*state).index(),
            index < 64,
            bit64_get((*state).word(), index),
        ensures
            (*next).index() == (*state).index() + 1,
            (*next).word() == (*state).word(),
    {
        println!("executing bit {index}");
        unimplemented!()
    }
}

mod implementation {
    #[allow(unused_imports)]
    use builtin::*;
    #[allow(unused_imports)]
    use builtin_macros::*;
    #[allow(unused_imports)]
    use crate::pervasive::{*, vec::*, seq::*, modes::*};
    use crate::trusted::*;

    pub proof fn skip_first_3_bits(tracked state: State) -> (tracked last: State)
        requires
            state.word() & 7 == 0,
            state.index() == 0,
        ensures
            last.index() == 3,
    {
        let word = state.word();
        assert(word & 7 == 0 ==> (word >> 0u64) & 1 == 0) by(bit_vector);
        assert(word & 7 == 0 ==> (word >> 1u64) & 1 == 0) by(bit_vector);
        assert(word & 7 == 0 ==> (word >> 2u64) & 1 == 0) by(bit_vector);
        let tracked state = skip_bit(tracked state);
        let tracked state = skip_bit(tracked state);
        let tracked state = skip_bit(tracked state);
        tracked state
    }

    pub fn execute_bits(word: u64, state: Tracked<State>) -> (last: Tracked<State>)
        requires
            word == (*state).word(),
            (*state).index() == 0,
        ensures
            (*last).index() == 3,
    {
        // bits 0, 1, 2
        if word & 7 == 0 {
            return tracked(skip_first_3_bits(state.get()));
        }

        // bit 0
        assert(word >> 0u64 == word) by(bit_vector);
        let state = if word & 1 == 1 {
            execute_bit(0, state)
        } else {
            tracked(skip_bit(state.get()))
        };

        // bit 1
        let state = if (word >> 1) & 1 == 1 {
            execute_bit(1, state)
        } else {
            tracked(skip_bit(state.get()))
        };

        // bit 2
        let state = if (word >> 2) & 1 == 1 {
            execute_bit(2, state)
        } else {
            tracked(skip_bit(state.get()))
        };

        state
    }
}

}
