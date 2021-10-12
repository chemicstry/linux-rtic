//! Real-Time Interrupt-driven Concurrency (RTIC) framework for real-time Linux
//!
//! **IMPORTANT**: This crate is published as [`linux-rtic`] on crates.io but the name of the
//! library is `rtic`.
//!
//! [`linux-rtic`]: https://crates.io/crates/linux-rtic
//!
//! The user level documentation is limited, but can be found in README and examples.
//!

pub use ctrlc;
pub use futex_queue as mpsc;
pub use lazy_static;
pub use linux_rtic_macros::app;
pub use pcp_mutex::PcpMutex;
pub use rtic_core::{prelude as mutex_prelude, Exclusive, Mutex};

use std::cell::UnsafeCell;

#[cfg(feature = "profiling")]
pub use tracing;
#[cfg(feature = "profiling")]
pub use tracing_chrome;
#[cfg(feature = "profiling")]
pub use tracing_subscriber;

pub mod slab;

pub fn init_thread_state(priority: pcp_mutex::Priority) {
    #[cfg(feature = "rt")]
    pcp_mutex::thread::init_fifo_priority(priority).expect("Error setting thread priority");
}

/// Internal replacement for `static mut T`
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    /// Create a RacyCell
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get `&mut T`
    #[inline(always)]
    pub unsafe fn get_mut_unchecked(&self) -> &mut T {
        &mut *self.0.get()
    }

    /// Get `&T`
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}
