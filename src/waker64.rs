// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use ::std::{cell::UnsafeCell, mem};

//==============================================================================
// Structures
//==============================================================================

/// 64-Bit Waker
pub struct Waker64(UnsafeCell<u64>);

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for 64-Bit Wakers
impl Waker64 {
    /// Creates a 64-Bit Waker from `val`.
    pub fn new(val: u64) -> Self {
        Waker64(UnsafeCell::new(val))
    }

    /// Applies the OR operator between `val` and the target [Waker64].
    /// The resulting value is stored back in the target [Waker64].
    pub fn fetch_or(&self, val: u64) {
        let s = unsafe { &mut *self.0.get() };
        *s |= val;
    }

    /// Applies the AND operator between `val` and the target [Waker64].
    /// The resulting value is stored back in the target [Waker64].
    pub fn fetch_and(&self, val: u64) {
        let s = unsafe { &mut *self.0.get() };
        *s &= val;
    }

    /// Applies the ADD operator between `val` and the target [Waker64].
    /// The resulting value is stored back in the target [Waker64] and the old
    /// value is returned.
    pub fn fetch_add(&self, val: u64) -> u64 {
        let s = unsafe { &mut *self.0.get() };
        let old = *s;
        *s += val;
        old
    }

    /// Applies the SUB operator between `val` and the target [Waker64].
    /// The resulting value is stored back in the target [Waker64] and the old
    /// value is returned.
    pub fn fetch_sub(&self, val: u64) -> u64 {
        let s = unsafe { &mut *self.0.get() };
        let old = *s;
        *s -= val;
        old
    }

    /// Returns the value stored in the the target [Waker64].
    pub fn load(&self) -> u64 {
        let s = unsafe { &mut *self.0.get() };
        *s
    }

    /// Replaces the value stored in the the target [Waker64] by `val`.
    pub fn swap(&self, val: u64) -> u64 {
        let s = unsafe { &mut *self.0.get() };
        mem::replace(s, val)
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

/// Sync Trait Implementation for 64-Bit Wakers
unsafe impl Sync for Waker64 {}
