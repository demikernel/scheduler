// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use crate::waker64::Waker64;

//==============================================================================
// Constants
//==============================================================================

/// Number of Bits in a [Waker64]
///
/// TODO: Move this to [crate::waker64].
pub const WAKER_BIT_LENGTH: usize = 64;

/// Size of Pages (in bytes)
pub const WAKER_PAGE_SIZE: usize = 64;

//==============================================================================
// Structures
//==============================================================================

/// Waker Page
///
/// This structure holds the status of multiple futures in the scheduler. It is
/// composed by 3 bitmaps, each of which having the ith bit to represent some
/// state for the ith future.
///
/// The number of bytes in this structure should match the number of bits in a
/// [Waker64]. Furthermore, the structure should be aligned in memory with its
/// own size. We rely on these two properties to distribute raw pointers to the
/// scheduler, so that it may cast back a raw pointer and operate on a specific
/// future whenever needed.
///
/// TODO: use the unused space in this structure to something useful.
#[repr(align(64))]
pub struct WakerPage {
    /// Reference count for the page.
    refcount: Waker64,
    /// Flags wether or not a given future has been notified.
    notified: Waker64,
    /// Flags whether or not a given future has completed.
    completed: Waker64,
    /// Flags whether or not a given future has ben dropped.
    dropped: Waker64,
    /// Padding required to make the structure 64-byte big.
    _unused: [u8; 32],
}

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for Waker Page
impl WakerPage {
    /// Sets the notification flag for the `ix` future in the target [WakerPage].
    pub fn notify(&self, ix: usize) {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        self.notified.fetch_or(1 << ix);
    }

    /// Takes out notification flags in the target [WakerPage].
    /// Notification flags are reset after this operation.
    pub fn take_notified(&self) -> u64 {
        // Unset all completed bits, since spurious notifications for completed
        // futures would lead us to poll them after completion.
        let mut notified = self.notified.swap(0);
        notified &= !self.completed.load();
        notified &= !self.dropped.load();
        notified
    }

    /// Queries whether or not the completed flag for the `ix` future in the target [WakerPage] is set.
    pub fn has_completed(&self, ix: usize) -> bool {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        self.completed.load() & (1 << ix) != 0
    }

    /// Sets the completed flag for the `ix` future in the target [WakerPage].
    pub fn mark_completed(&self, ix: usize) {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        self.completed.fetch_or(1 << ix);
    }

    /// Sets the dropped flag for the `ix` future in the target [WakerPage].
    pub fn mark_dropped(&self, ix: usize) {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        self.dropped.fetch_or(1 << ix);
    }

    /// Takes out dropped flags in the target [WakerPage].
    /// Dropped flags are reset after this operation.
    pub fn take_dropped(&self) -> u64 {
        self.dropped.swap(0)
    }

    /// Queries whether or not the dropped flag for the `ix` future in the target [WakerPage] is set.
    pub fn was_dropped(&self, ix: usize) -> bool {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        self.dropped.load() & (1 << ix) != 0
    }

    /// Resets all flags in the target [WakerPage].
    /// The reference count for the target page is reset to one.
    pub fn reset(&mut self) {
        self.refcount.swap(1);
        self.notified.swap(0);
        self.completed.swap(0);
        self.dropped.swap(0);
    }

    /// Initialize flags for the `ix` future in the target [WakerPage].
    /// Notification and dropped flags are reset after this operation.
    pub fn initialize(&self, ix: usize) {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        self.notified.fetch_or(1 << ix);
        self.completed.fetch_and(!(1 << ix));
        self.dropped.fetch_and(!(1 << ix));
    }

    /// Clears flags for the `ix` future in the target [WakerPage]
    /// The reference count for the target page is left unmodified.
    pub fn clear(&self, ix: usize) {
        debug_assert!(ix < WAKER_BIT_LENGTH);
        let mask: u64 = !(1 << ix);
        self.notified.fetch_and(mask);
        self.completed.fetch_and(mask);
        self.dropped.fetch_and(mask);
    }

    /// Increments the reference count of the target [WakerPage].
    /// The old reference count is returned.
    pub fn refcount_inc(&self) -> u64 {
        self.refcount.fetch_add(1)
    }

    /// Decrements the reference count of the target [WakerPage].
    /// The old reference count is returned.
    pub fn refcount_dec(&self) -> u64 {
        self.refcount.fetch_sub(1)
    }
}

//==============================================================================
// Unit Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use super::WakerPage;
    use super::WAKER_PAGE_SIZE;
    use std::mem;

    #[test]
    fn test_size() {
        assert_eq!(mem::size_of::<WakerPage>(), WAKER_PAGE_SIZE);
    }
}
