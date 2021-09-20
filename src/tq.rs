use crate::time::Instant;
use heapless::{binary_heap::Min, BinaryHeap};
use std::{cmp::Ordering, sync::Mutex};

pub struct TimerQueue<T: Copy, const N: usize> {
    queue: Mutex<BinaryHeap<NotReady<T>, Min, N>>,
    timer_fd: libc::c_int,
}

impl<T: Copy, const N: usize> TimerQueue<T, N> {
    pub fn new() -> Result<Self, i32> {
        let res = unsafe { libc::timerfd_create(libc::CLOCK_MONOTONIC, 0) };

        if res > 0 {
            Ok(Self {
                queue: Mutex::new(BinaryHeap::default()),
                timer_fd: res,
            })
        } else {
            Err(res)
        }
    }

    pub fn enqueue(&self, nr: NotReady<T>) -> Result<(), NotReady<T>> {
        let mut queue = self.queue.lock().unwrap();

        let rearm = if queue
            .peek()
            .map(|head| nr.instant < head.instant)
            .unwrap_or(true)
        {
            Some(nr.instant)
        } else {
            None
        };

        match queue.push(nr) {
            Ok(_) => {
                if let Some(instant) = rearm {
                    self.set(instant).expect("Error setting timer");
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
                self.set(nr.instant).expect("Error setting timer");
                None
            }
        } else {
            None
        }
    }

    /// Blocks the current thread until the timer expires
    pub fn wait(&self) {
        let mut buf: u64 = 0;
        unsafe {
            libc::read(
                self.timer_fd,
                (&mut buf as *mut u64) as *mut libc::c_void,
                core::mem::size_of::<u64>(),
            );
        }
    }

    fn set(&self, instant: Instant) -> Result<(), i32> {
        let itimerspec = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: instant.into(),
        };

        let res = unsafe {
            libc::timerfd_settime(
                self.timer_fd,
                libc::TFD_TIMER_ABSTIME,
                &itimerspec,
                0 as *mut libc::itimerspec,
            )
        };

        if res == 0 {
            Ok(())
        } else {
            Err(res)
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
