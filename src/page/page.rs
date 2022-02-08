// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! A "page" is made up of 64 contiguous entries, each entry represents the state of a task/future
//! in our scheduler. This page is represented by a 64-bit integer where the ith bit corresponds to
//! the ith task in that page. This way fast bit arithmetic can be used to index into a task's
//! state and uniquely identify a task among multiple pages.

//==============================================================================
// Imports
//==============================================================================

use crate::waker64::WakerU64;

//==============================================================================
// Constants
//==============================================================================

/// Size of our pages. Should be the same size as the number of bits in the underlying data type
/// representing our bit vectors.
pub const WAKER_PAGE_SIZE: usize = 64;

//==============================================================================
// Structures
//==============================================================================

/// A page is used by the scheduler to hold the current status of 64 different futures in the
/// scheduler. So we use 64bit integers where the ith bit represents the ith future. Pages are
/// arranged by the scheduler in a `pages` vector of pages which grows as needed allocating space
/// for 64 more futures at a time.
#[repr(align(64))]
pub struct WakerPage {
    /// We use a single bit for our reference count implying only reference exists per future
    /// at a time.
    pub refcount: WakerU64,
    /// A 64 element bit vector representing the futures for this page which have been notified
    /// by a wake and are ready to be polled again. The ith bit represents the ith future in the
    /// corresponding memory slab.
    pub notified: WakerU64,
    pub completed: WakerU64,
    pub dropped: WakerU64,
    _unused: [u8; 32],
}

//==============================================================================
// Associate Functions
//==============================================================================

impl WakerPage {
    pub fn notify(&self, ix: usize) {
        debug_assert!(ix < 64);
        self.notified.fetch_or(1 << ix);
    }

    /// Return a bit vector representing the futures in this page which are ready to be
    /// polled again.
    pub fn take_notified(&self) -> u64 {
        // Unset all ready bits, since spurious notifications for completed futures would lead
        // us to poll them after completion.
        let mut notified = self.notified.swap(0);
        notified &= !self.completed.load();
        notified &= !self.dropped.load();
        notified
    }

    pub fn has_completed(&self, ix: usize) -> bool {
        debug_assert!(ix < 64);
        self.completed.load() & (1 << ix) != 0
    }

    pub fn mark_completed(&self, ix: usize) {
        debug_assert!(ix < 64);
        self.completed.fetch_or(1 << ix);
    }

    pub fn mark_dropped(&self, ix: usize) {
        debug_assert!(ix < 64);
        self.dropped.fetch_or(1 << ix);
    }

    pub fn take_dropped(&self) -> u64 {
        self.dropped.swap(0)
    }

    pub fn was_dropped(&self, ix: usize) -> bool {
        debug_assert!(ix < 64);
        self.dropped.load() & (1 << ix) != 0
    }

    pub fn initialize(&self, ix: usize) {
        debug_assert!(ix < 64);
        self.notified.fetch_or(1 << ix);
        self.completed.fetch_and(!(1 << ix));
        self.dropped.fetch_and(!(1 << ix));
    }

    pub fn clear(&self, ix: usize) {
        debug_assert!(ix < 64);
        let mask = !(1 << ix);
        self.notified.fetch_and(mask);
        self.completed.fetch_and(mask);
        self.dropped.fetch_and(mask);
    }
}
