// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! A "page" is made up of 64 contiguous entries, each entry represents the state of a task/future
//! in our scheduler. This page is represented by a 64-bit integer where the ith bit corresponds to
//! the ith task in that page. This way fast bit arithmetic can be used to index into a task's
//! state and uniquely identify a task among multiple pages.

//==============================================================================
// Imports
//==============================================================================

use crate::{
    page::{WakerPage, WAKER_PAGE_SIZE},
    waker64::Waker64,
};
use ::std::{
    alloc::{Allocator, Global, Layout},
    mem,
    ops::Deref,
    ptr::{self, NonNull},
};

//==============================================================================
// Structures
//==============================================================================

/// Waker Page Reference
pub struct WakerPageRef(NonNull<WakerPage>);

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for Waker Page Reference
impl WakerPageRef {
    pub fn new(waker_page: NonNull<WakerPage>) -> Self {
        Self { 0: waker_page }
    }

    /// Creates a new waker using the local index and our WakerPage reference.
    pub fn into_raw_waker_ref(&self, ix: usize) -> NonNull<u8> {
        debug_assert!(ix < WAKER_PAGE_SIZE);

        // Bump the refcount for our new reference.
        let self_: WakerPageRef = self.clone();
        mem::forget(self_);

        unsafe {
            let base_ptr: *mut u8 = self.0.as_ptr().cast();
            let ptr: NonNull<u8> = NonNull::new_unchecked(base_ptr.add(ix));
            ptr
        }
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

/// Clone Trait Implementation for Waker Page References
impl Clone for WakerPageRef {
    fn clone(&self) -> Self {
        let old_refount: u64 = unsafe { self.0.as_ref().refcount.fetch_add(1) };
        debug_assert!(old_refount < std::u64::MAX);
        Self(self.0)
    }
}

/// Drop Trait Implementation for Waker Page References
impl Drop for WakerPageRef {
    fn drop(&mut self) {
        unsafe {
            if self.0.as_ref().refcount.fetch_sub(1) != 1 {
                return;
            }
            ptr::drop_in_place(self.0.as_mut());
            Global.deallocate(self.0.cast(), Layout::for_value(self.0.as_ref()));
        }
    }
}

/// De-Reference Trait Implementation for Waker Page References
impl Deref for WakerPageRef {
    type Target = WakerPage;

    fn deref(&self) -> &WakerPage {
        unsafe { self.0.as_ref() }
    }
}

/// Default Trait Implementation for Waker Page References
impl Default for WakerPageRef {
    fn default() -> Self {
        let layout = Layout::new::<WakerPage>();
        assert_eq!(layout.align(), WAKER_PAGE_SIZE);
        let mut ptr: NonNull<WakerPage> =
            Global.allocate(layout).expect("Allocation failed").cast();
        unsafe {
            let page: &mut WakerPage = ptr.as_mut();
            ptr::write(&mut page.refcount as *mut _, Waker64::new(1));
            ptr::write(&mut page.notified as *mut _, Waker64::new(0));
            ptr::write(&mut page.completed as *mut _, Waker64::new(0));
            ptr::write(&mut page.dropped as *mut _, Waker64::new(0));
        }
        Self(ptr)
    }
}
