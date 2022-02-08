// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Implementation of our efficient, single-threaded task scheduler.
//!
//! Our scheduler holds [SchedulerFuture]s in a memory slab for short
//! lived tasks on in the heap for longer lived tasks. Notice the slab is pinned memory as we must
//! make sure Futures don't move (Pin). The scheduler is a single-threaded runtime which attempts
//! to run all tasks which are available to run again. The Background tasks are always ready to poll
//! again.
//!
//! As background tasks are polled, they notify task in our scheduler via the WakerPage mechanism
//! so the scheduler only polls (schedules and runs) tasks that it knows are ready to run.

//==============================================================================
// Imports
//==============================================================================

use crate::{
    pin_slab::PinSlab,
    waker_page::{WakerPage, WakerPageRef, WAKER_PAGE_SIZE},
};
use ::bit_iter::BitIter;
use ::std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

//==============================================================================
// Exports
//==============================================================================

pub use super::{future::SchedulerFuture, handle::SchedulerHandle};

//==============================================================================
// Structures
//==============================================================================

/// Actual data used by [Scheduler].
struct Inner<F: Future<Output = ()> + Unpin> {
    /// Tasks are held by the scheduler in this memory slab.
    slab: PinSlab<F>,
    /// Holds the current status of which tasks are ready to be polled (scheduled) again.
    /// The statuses are arranged in pages.
    pages: Vec<WakerPageRef>,
}

/// Future Scheduler
#[derive(Clone)]
pub struct Scheduler {
    inner: Rc<RefCell<Inner<Box<dyn SchedulerFuture>>>>,
}

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for Inner
impl<F: Future<Output = ()> + Unpin> Inner<F> {
    /// Our pages hold 64 contiguous future wakers, so we can do simple arithmetic to access the
    /// correct page as well as the index within page.
    /// Given the `key` representing a future, return a reference to that page, `WakerPageRef`. And
    /// the index _within_ that page (usize).
    fn page(&self, key: u64) -> (&WakerPageRef, usize) {
        let key = key as usize;
        let (page_ix, subpage_ix) = (key / WAKER_PAGE_SIZE, key % WAKER_PAGE_SIZE);
        (&self.pages[page_ix], subpage_ix)
    }

    /// Insert a future into our scheduler returning an integer key representing this future. This
    /// key is used to index into the slab for accessing the future.
    fn insert(&mut self, future: F) -> u64 {
        let key = self.slab.insert(future);

        // Add a new page to hold this future's status if the current page is filled.
        while key >= self.pages.len() * WAKER_PAGE_SIZE {
            self.pages.push(WakerPage::new());
        }
        let (page, subpage_ix) = self.page(key as u64);
        page.initialize(subpage_ix);
        key as u64
    }
}

/// Associate Functions for Future Schedulers
impl Scheduler {
    /// Given a handle representing a future, remove the future from the scheduler returning it.
    pub fn take(&self, mut handle: SchedulerHandle) -> Box<dyn SchedulerFuture> {
        let mut inner = self.inner.borrow_mut();
        let key = handle.take_key().unwrap();
        let (page, subpage_ix) = inner.page(key);
        assert!(!page.was_dropped(subpage_ix));
        page.clear(subpage_ix);
        inner.slab.remove_unpin(key as usize).unwrap()
    }

    /// Given the raw `key` representing this future return a proper handle.
    pub fn from_raw_handle(&self, key: u64) -> Option<SchedulerHandle> {
        let inner = self.inner.borrow();
        inner.slab.get(key as usize)?;
        let (page, _) = inner.page(key);
        let handle = SchedulerHandle::new(key, page.clone());
        Some(handle)
    }

    /// Insert a new task into our scheduler returning a handle corresponding to it.
    pub fn insert<F: SchedulerFuture>(&self, future: F) -> SchedulerHandle {
        let mut inner = self.inner.borrow_mut();
        let key = inner.insert(Box::new(future));
        let (page, _) = inner.page(key);
        SchedulerHandle::new(key, page.clone())
    }

    /// Poll all futures which are ready to run again. Tasks in our scheduler are notified when
    /// relevant data or events happen. The relevant event have callback function (the waker) which
    /// they can invoke to notify the scheduler that future should be polled again.
    pub fn poll(&self) {
        let mut inner = self.inner.borrow_mut();
        // inner.root_waker.register(ctx.waker());

        // TODO: rewrite this loop to use high-level iterators instead of indexes.
        // Iterate through all our pages finding the tasks that are ready to be polled again
        // (notified) and dropped tasks which can be removed.
        for page_ix in 0..inner.pages.len() {
            let (notified, dropped) = {
                let page = &mut inner.pages[page_ix];
                (page.take_notified(), page.take_dropped())
            };
            // Non-zero means at least one future in this page should be polled.
            if notified != 0 {
                // Iterate through this page's bit vector polling the futures that are ready.
                for subpage_ix in BitIter::from(notified) {
                    if subpage_ix != 0 {
                        // Get future using our page indices and poll it!
                        let ix = page_ix * WAKER_PAGE_SIZE + subpage_ix;
                        let waker =
                            unsafe { Waker::from_raw(inner.pages[page_ix].raw_waker(subpage_ix)) };
                        let mut sub_ctx = Context::from_waker(&waker);

                        let pinned_ref = inner.slab.get_pin_mut(ix).unwrap();
                        let pinned_ptr = unsafe { Pin::into_inner_unchecked(pinned_ref) as *mut _ };

                        drop(inner);
                        let pinned_ref = unsafe { Pin::new_unchecked(&mut *pinned_ptr) };
                        let poll_result = { Future::poll(pinned_ref, &mut sub_ctx) };
                        inner = self.inner.borrow_mut();

                        match poll_result {
                            Poll::Ready(()) => inner.pages[page_ix].mark_completed(subpage_ix),
                            Poll::Pending => (),
                        }
                    }
                }
            }
            if dropped != 0 {
                for subpage_ix in BitIter::from(dropped) {
                    if subpage_ix != 0 {
                        let ix = page_ix * WAKER_PAGE_SIZE + subpage_ix;
                        inner.slab.remove(ix);
                        inner.pages[page_ix].clear(subpage_ix);
                    }
                }
            }
        }
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

/// Default Trait IMplementation for Future SchedulerS
impl Default for Scheduler {
    /// Creates a scheduler with default values.
    fn default() -> Self {
        let inner = Inner {
            slab: PinSlab::new(),
            pages: vec![],
        };
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }
}
