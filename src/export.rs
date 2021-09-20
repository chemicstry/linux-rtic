pub use lazy_static;

// Priority Ceiling Protocol mutexes
pub use pcp_mutex::PcpManager as MutexManager;
pub use pcp_mutex::PcpMutex as Mutex;

pub use flume as mpmc;

use std::cell::Cell;

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
    #[allow(dead_code)]
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

#[inline(always)]
pub fn set_current_thread_priority(priority: u8) -> Result<(), i32> {
    let param = libc::sched_param {
        sched_priority: priority as i32,
    };

    // sched_setparam calls sched_setscheduler internally, so there is no overhead of specifying redundant policy
    let res = unsafe { libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) };

    if res != 0 {
        Err(res)
    } else {
        Ok(())
    }
}

/// Lock the resource proxy by setting the BASEPRI
/// and running the closure with interrupt::free
#[allow(unused_variables)]
#[inline(always)]
pub fn lock<T, R>(
    res: &Mutex<T>,
    priority: &Priority,
    ceiling: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    let current = priority.get();

    #[cfg(feature = "use_srp")]
    {
        priority.set(ceiling);
        set_current_thread_priority(ceiling)
            .expect("Failed to set thread priority. Insufficient permissions?");
    }
    // Note that we lock with previous priority as PcpMutex will calculate ceiling itself
    let r = f(&mut res.lock(current));
    #[cfg(feature = "use_srp")]
    {
        set_current_thread_priority(current)
            .expect("Failed to set thread priority. Insufficient permissions?");
        priority.set(current);
    }

    r
}
