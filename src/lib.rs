// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy::all)]
#![recursion_limit = "512"]
#![feature(test)]
#![feature(allocator_api)]

mod page;
mod pin_slab;
mod scheduler;
mod waker64;

//==============================================================================
// Imports
//==============================================================================

extern crate test;

//==============================================================================
// Exports
//==============================================================================

pub use scheduler::{
    FutureResult,
    Scheduler,
    SchedulerFuture,
    SchedulerHandle,
};
