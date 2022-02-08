// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! A "page" is made up of 64 contiguous entries, each entry represents the state of a task/future
//! in our scheduler. This page is represented by a 64-bit integer where the ith bit corresponds to
//! the ith task in that page. This way fast bit arithmetic can be used to index into a task's
//! state and uniquely identify a task among multiple pages.

//==============================================================================
// Imports
//==============================================================================

use super::page::WakerPage;
use crate::waker64::Waker64;
use ::std::{
    alloc::{Allocator, Global, Layout},
    mem,
    ops::Deref,
    ptr::{self, NonNull},
    task::{RawWaker, RawWakerVTable},
};

//==============================================================================
// Structures
//==============================================================================

pub struct WakerPageRef(NonNull<WakerPage>);

#[repr(transparent)]
struct WakerRef(NonNull<u8>);

//==============================================================================
// Associate Functions
//==============================================================================

impl WakerPageRef {
    pub fn new() -> Self {
        let layout = Layout::new::<WakerPage>();
        assert_eq!(layout.align(), 64);
        let mut ptr: NonNull<WakerPage> =
            Global.allocate(layout).expect("Allocation failed").cast();
        unsafe {
            let page = ptr.as_mut();
            ptr::write(&mut page.refcount as *mut _, Waker64::new(1));
            ptr::write(&mut page.notified as *mut _, Waker64::new(0));
            ptr::write(&mut page.completed as *mut _, Waker64::new(0));
            ptr::write(&mut page.dropped as *mut _, Waker64::new(0));
        }
        Self(ptr)
    }

    /// Get the waker for the future at the `local_index` location on this page.
    /// 0 <= `local_index` <= 64
    pub fn raw_waker(&self, local_index: usize) -> RawWaker {
        self.waker(local_index).into_raw_waker()
    }

    /// Create a new waker using the local index and our WakerPage reference.
    fn waker(&self, ix: usize) -> WakerRef {
        debug_assert!(ix < 64);

        // Bump the refcount for our new reference.
        let self_ = self.clone();
        mem::forget(self_);

        unsafe {
            let base_ptr: *mut u8 = self.0.as_ptr().cast();
            let ptr = NonNull::new_unchecked(base_ptr.add(ix));
            WakerRef(ptr)
        }
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

impl Clone for WakerPageRef {
    fn clone(&self) -> Self {
        let new_refcount = unsafe {
            // TODO: We could use `Relaxed` here, see `std::sync::Arc` for documentation.
            self.0.as_ref().refcount.fetch_add(1)
        };
        debug_assert!(new_refcount < std::isize::MAX as u64);
        Self(self.0)
    }
}

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

impl Deref for WakerPageRef {
    type Target = WakerPage;

    fn deref(&self) -> &WakerPage {
        unsafe { self.0.as_ref() }
    }
}

impl WakerRef {
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

    fn into_raw_waker(self) -> RawWaker {
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
    q.into_raw_waker()
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

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    waker_ref_clone,
    waker_ref_wake,
    waker_ref_wake_by_ref,
    waker_ref_drop,
);

impl Clone for WakerRef {
    fn clone(&self) -> Self {
        let (base_ptr, _) = self.base_ptr();
        let p = WakerPageRef(base_ptr);
        mem::forget(p.clone());
        mem::forget(p);
        WakerRef(self.0)
    }
}

impl Drop for WakerRef {
    fn drop(&mut self) {
        let (base_ptr, _) = self.base_ptr();
        // Decrement the refcount.
        drop(WakerPageRef(base_ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::super::page::WakerPage;
    use super::WakerPageRef;
    use std::mem;

    #[test]
    fn test_size() {
        assert_eq!(mem::size_of::<WakerPage>(), 64);
    }

    #[test]
    fn test_basic() {
        let p = WakerPageRef::new();

        let q = p.waker(0);
        let r = p.waker(63);
        let s = p.waker(16);

        q.wake();
        r.wake();

        assert_eq!(p.take_notified(), 1 << 0 | 1 << 63);

        s.wake();

        assert_eq!(p.take_notified(), 1 << 16);
    }
}
