// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use ::std::{cell::UnsafeCell, rc::Rc, task::Waker};

//==============================================================================
// Structures
//==============================================================================

struct WakerSlot(UnsafeCell<Option<Waker>>);

pub struct SharedWaker(Rc<WakerSlot>);

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for Shared Waker
impl SharedWaker {
    /// Wakes up the task that is associated with the target [SharedWaker].
    pub fn wake(&self) {
        let s: &mut Option<Waker> = unsafe {
            let waker: &Rc<WakerSlot> = &self.0;
            let cell: &UnsafeCell<Option<Waker>> = &waker.0;
            &mut *cell.get()
        };
        if let Some(waker) = s.take() {
            waker.wake();
        }
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

/// Send Trait Implementation for Waker Slots
unsafe impl Send for WakerSlot {}

/// Sync Trait Implementation for Wake Slots
unsafe impl Sync for WakerSlot {}

/// Clone Trait Implementation for Shared Wakers
impl Clone for SharedWaker {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Default Trait Implementation for Shared Wakers
impl Default for SharedWaker {
    /// Creates a [SharedWaker] with default values.
    fn default() -> Self {
        Self(Rc::new(WakerSlot(UnsafeCell::new(None))))
    }
}
