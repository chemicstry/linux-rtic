use crate::time::Instant;
use heapless::{binary_heap::Min, BinaryHeap};
use std::{cmp::Ordering, sync::{Mutex, atomic::{self, AtomicI32}}};

pub struct TimerQueue<T: Copy, const N: usize> {
    queue: Mutex<BinaryHeap<NotReady<T>, Min, N>>,
    waiting_thread: AtomicI32,
}

impl<T: Copy, const N: usize> TimerQueue<T, N> {
    pub fn new() -> Self {
        Self {
                queue: Mutex::new(BinaryHeap::default()),
                waiting_thread: Default::default(),
            }
    }

    pub fn enqueue(&self, nr: NotReady<T>) -> Result<(), NotReady<T>> {
        let mut queue = self.queue.lock().unwrap();

        let rearm = queue.peek()
            .map(|head| nr.instant < head.instant)
            .unwrap_or(true);

        match queue.push(nr) {
            Ok(_) => {
                if rearm {
                    self.wake();
                }

                Ok(())
            }
            Err(nr) => Err(nr),
        }
    }

    pub fn dequeue(&self) -> Option<NotReady<T>> {
        let mut queue = self.queue.lock().unwrap();

        if let Some(nr) = queue.peek() {
            if nr.instant <= Instant::now() {
                Some(unsafe { queue.pop_unchecked() })
            } else {
                None
            }
        } else {
            None
        }
    }

    // Wakes the waiting thread, blocked on wait function
    pub fn wake(&self) {
        let thread = self.waiting_thread.load(atomic::Ordering::Relaxed);
        if thread != 0 {
            unsafe {
                libc::kill(thread, libc::SIGUSR1);
            }
        }
    }

    /// Blocks the current thread until the timer expires
    pub fn wait(&self) {
        if self.waiting_thread.load(atomic::Ordering::Relaxed) == 0 {
            let pid = unsafe { libc::getpid() };
            self.waiting_thread.store(pid, atomic::Ordering::Relaxed);
        }

        let instant = self.queue.lock().unwrap().peek().map(|r| r.instant.into()).unwrap_or(libc::timespec {
            // If there are no active timers, sleep until about December 4th, 292,277,026,596 AD 20:10:55 UTC
            tv_sec: i64::MAX,
            tv_nsec: 0,
        });

        unsafe {
            libc::clock_nanosleep(libc::CLOCK_MONOTONIC, libc::TIMER_ABSTIME, &instant.into(), 0 as _);
        }
    }
}

pub struct NotReady<T>
where
    T: Copy,
{
    pub handle: crate::slab::SlabHandle,
    pub instant: Instant,
    pub task: T,
}

impl<T> Eq for NotReady<T> where T: Copy {}

impl<T> Ord for NotReady<T>
where
    T: Copy,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<T> PartialEq for NotReady<T>
where
    T: Copy,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<T> PartialOrd for NotReady<T>
where
    T: Copy,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
