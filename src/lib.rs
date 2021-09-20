pub use ctrlc;
pub use linux_rtic_macros::app;
pub use rtic_core::{prelude as mutex_prelude, Exclusive, Mutex};
use std::cell::UnsafeCell;

#[cfg(feature = "profiling")]
pub use tracing;

/// Contains everything that is internally used after macro expansion
pub mod export;

pub mod slab;

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
