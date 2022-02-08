// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! A "page" is made up of 64 contiguous entries, each entry represents the state of a task/future
//! in our scheduler. This page is represented by a 64-bit integer where the ith bit corresponds to
//! the ith task in that page. This way fast bit arithmetic can be used to index into a task's
//! state and uniquely identify a task among multiple pages.

//==============================================================================
// Imports
//==============================================================================

use super::{page::WakerPage, WakerPageRef};
use ::std::{
    mem,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable},
};

//==============================================================================
// Structures
//==============================================================================

#[repr(transparent)]
pub struct WakerRef(NonNull<u8>);

//==============================================================================
// Associate Functions
//==============================================================================

impl WakerRef {
    pub fn new(raw_page_ref: NonNull<u8>) -> Self {
        Self { 0: raw_page_ref }
    }

    fn base_ptr(&self) -> (NonNull<WakerPage>, usize) {
        let ptr = self.0.as_ptr();

        let forward_offset = ptr.align_offset(64);
        let mut base_ptr = ptr;
        let mut offset = 0;
        if forward_offset != 0 {
            offset = 64 - forward_offset;
            base_ptr = ptr.wrapping_sub(offset);
        }
        unsafe { (NonNull::new_unchecked(base_ptr).cast(), offset) }
    }

    fn wake_by_ref(&self) {
        let (base_ptr, offset) = self.base_ptr();
        let base = unsafe { &*base_ptr.as_ptr() };
        base.notify(offset);
    }

    fn wake(self) {
        self.wake_by_ref()
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

impl Into<RawWaker> for WakerRef {
    fn into(self) -> RawWaker {
        let ptr = self.0.cast().as_ptr() as *const ();
        let waker = RawWaker::new(ptr, &VTABLE);
        mem::forget(self);
        waker
    }
}

// The following methods are used to implement Waker for WakeRef.
//
// While it may look complicated it is just doing standard implementation of the Waker vtable as
// required by Rust.
//
// Ultimately, calling .wake() for our waker just calls [WakerPage::notify] which sets the
// appropriate bit to 1.

unsafe fn waker_ref_clone(ptr: *const ()) -> RawWaker {
    let p = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    let q = p.clone();
    mem::forget(p);
    q.into()
}

unsafe fn waker_ref_wake(ptr: *const ()) {
    let p = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    p.wake();
}

unsafe fn waker_ref_wake_by_ref(ptr: *const ()) {
    let p = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    p.wake_by_ref();
    mem::forget(p);
}

unsafe fn waker_ref_drop(ptr: *const ()) {
    let p = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    drop(p);
}

pub const VTABLE: RawWakerVTable = RawWakerVTable::new(
    waker_ref_clone,
    waker_ref_wake,
    waker_ref_wake_by_ref,
    waker_ref_drop,
);

impl Clone for WakerRef {
    fn clone(&self) -> Self {
        let (base_ptr, _) = self.base_ptr();
        let p = WakerPageRef::new(base_ptr);
        mem::forget(p.clone());
        mem::forget(p);
        WakerRef(self.0)
    }
}

impl Drop for WakerRef {
    fn drop(&mut self) {
        let (base_ptr, _) = self.base_ptr();
        // Decrement the refcount.
        drop(WakerPageRef::new(base_ptr));
    }
}

//==============================================================================
// Unit Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use crate::page::WakerPageRef;
    use crate::page::WakerRef;

    #[test]
    fn test_basic() {
        let p = WakerPageRef::default();

        let q = WakerRef::new(p.into_raw_waker_ref(0));
        let r = WakerRef::new(p.into_raw_waker_ref(31));
        let s = WakerRef::new(p.into_raw_waker_ref(15));

        q.wake();
        r.wake();

        assert_eq!(p.take_notified(), 1 << 0 | 1 << 31);

        s.wake();

        assert_eq!(p.take_notified(), 1 << 15);
    }
}
