use heapless::mpmc::MpMcQueue;
pub use lazy_static::lazy_static;
use std::cell::Cell;

// Priority Ceiling Protocol mutexes
pub use pcp_mutex::PcpMutex as Mutex;
pub use pcp_mutex::PcpManager as MutexManager;

// Queue that holds inputs of a single task
pub type TaskInputQueue<T, const N: usize> = MpMcQueue<T, N>;

// Newtype over `Cell` that forbids mutation through a shared reference
pub struct Priority {
    inner: Cell<u8>,
}

impl Priority {
    /// Create a new Priority
    ///
    /// # Safety
    ///
    /// Will overwrite the current Priority
    #[inline(always)]
    pub unsafe fn new(value: u8) -> Self {
        Priority {
            inner: Cell::new(value),
        }
    }

    /// Change the current priority to `value`
    // These two methods are used by `lock` (see below) but can't be used from the RTIC application
    #[inline(always)]
    fn set(&self, value: u8) {
        self.inner.set(value)
    }

    /// Get the current priority
    #[inline(always)]
    fn get(&self) -> u8 {
        self.inner.get()
    }
}

/// Lock the resource proxy by setting the BASEPRI
/// and running the closure with interrupt::free
#[inline(always)]
pub fn lock<T, R>(
    res: &Mutex<T>,
    priority: &Priority,
    _ceiling: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    let current = priority.get();

    // TODO: set priority to ceiling
    let r = f(&mut res.lock(current));
    // TODO: set priority to current

    r
}
