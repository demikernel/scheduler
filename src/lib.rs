// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy::all)]
#![recursion_limit = "512"]
#![feature(test)]
#![feature(allocator_api)]

mod pin_slab;
mod scheduler;
mod waker64;
mod waker_page;

//==============================================================================
// Exports
//==============================================================================

pub use scheduler::{Scheduler, SchedulerFuture, SchedulerHandle};
