// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use super::{page::WakerPage, WakerPageRef, WAKER_PAGE_SIZE};
use ::std::{
    mem,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable},
};

//==============================================================================
// Structures
//==============================================================================

/// Waker Reference
///
/// This reference is a representation for the status of a particular task
/// stored in a [WakerPage].
#[repr(transparent)]
pub struct WakerRef(NonNull<u8>);

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for Waker References
impl WakerRef {
    pub fn new(raw_page_ref: NonNull<u8>) -> Self {
        Self(raw_page_ref)
    }

    /// Casts the target [WakerRef] back into reference to a [WakerPage] plus an
    /// offset indicating the target task in the latter structure.
    ///
    /// For more information on this hack see comments on [crate::page::WakerPageRef].
    fn base_ptr(&self) -> (NonNull<WakerPage>, usize) {
        let ptr: *mut u8 = self.0.as_ptr();
        let forward_offset: usize = ptr.align_offset(WAKER_PAGE_SIZE);
        let mut base_ptr: *mut u8 = ptr;
        let mut offset: usize = 0;
        if forward_offset != 0 {
            offset = WAKER_PAGE_SIZE - forward_offset;
            base_ptr = ptr.wrapping_sub(offset);
        }
        unsafe { (NonNull::new_unchecked(base_ptr).cast(), offset) }
    }

    /// Sets the notification flag for the task that associated with the target [WakerRef].
    fn wake_by_ref(&self) {
        let (base_ptr, ix): (NonNull<WakerPage>, usize) = self.base_ptr();
        let base: &WakerPage = unsafe { &*base_ptr.as_ptr() };
        base.notify(ix);
    }

    /// Sets the notification flag for the task that is associated with the target [WakerRef].
    fn wake(self) {
        self.wake_by_ref()
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

/// Clone Trait Implementation for Waker References
impl Clone for WakerRef {
    fn clone(&self) -> Self {
        let (base_ptr, _): (NonNull<WakerPage>, _) = self.base_ptr();
        let p: WakerPageRef = WakerPageRef::new(base_ptr);
        // Increment reference count.
        mem::forget(p.clone());
        // This is not a double increment.
        mem::forget(p);
        WakerRef(self.0)
    }
}

/// Drop Trait Implementation for Waker References
impl Drop for WakerRef {
    fn drop(&mut self) {
        let (base_ptr, _) = self.base_ptr();
        // Decrement the refcount.
        drop(WakerPageRef::new(base_ptr));
    }
}

/// Convert Trait Implementation for Waker References
impl Into<RawWaker> for WakerRef {
    fn into(self) -> RawWaker {
        let ptr: *const () = self.0.cast().as_ptr() as *const ();
        let waker: RawWaker = RawWaker::new(ptr, &VTABLE);
        // Increment reference count.
        mem::forget(self);
        waker
    }
}

/// Clones the task that is associated to the target [WakerRef].
unsafe fn waker_ref_clone(ptr: *const ()) -> RawWaker {
    let p: WakerRef = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    let q: WakerRef = p.clone();
    // Increment reference count.
    mem::forget(p);
    q.into()
}

/// Wakes up the task that is associated to the target [WakerRef].
unsafe fn waker_ref_wake(ptr: *const ()) {
    let p = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    p.wake();
}

/// Wakes up the task that is associated to the target [WakerRef].
unsafe fn waker_ref_wake_by_ref(ptr: *const ()) {
    let p: WakerRef = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    p.wake_by_ref();
    // Increment reference count.
    mem::forget(p);
}

/// Drops the task that is associated to the target [WakerRef].
unsafe fn waker_ref_drop(ptr: *const ()) {
    let p: WakerRef = WakerRef(NonNull::new_unchecked(ptr as *const u8 as *mut u8));
    // Decrement reference count.
    drop(p);
}

/// Raw Waker Trait Implementation for Waker References
pub const VTABLE: RawWakerVTable = RawWakerVTable::new(
    waker_ref_clone,
    waker_ref_wake,
    waker_ref_wake_by_ref,
    waker_ref_drop,
);

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
