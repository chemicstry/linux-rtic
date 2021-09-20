// Preallocated storage similar to slab crate, but fixed size

use std::{cell::UnsafeCell, mem::MaybeUninit};

use heapless::mpmc::MpMcQueue;

/// Handle to a queued item
#[derive(Debug)]
pub struct SlabHandle(usize);

pub struct Slab<T, const N: usize> {
    // Contains indices of free slots
    free_queue: MpMcQueue<SlabHandle, N>,
    slots: UnsafeCell<[MaybeUninit<T>; N]>,
}

unsafe impl<T: Send, const N: usize> Send for Slab<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for Slab<T, N> {}

impl<T, const N: usize> Slab<T, N> {
    pub fn new() -> Self {
        let free_queue = MpMcQueue::default();

        // Fill free queue
        (0..N).for_each(|i| free_queue.enqueue(SlabHandle(i)).unwrap());

        // Need unsafe, otherwise compiler complains about T not being copy
        let slots = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        Self {
            free_queue,
            slots: UnsafeCell::new(slots),
        }
    }

    pub fn insert(&self, item: T) -> Result<SlabHandle, T> {
        if let Some(handle) = self.free_queue.dequeue() {
            unsafe {
                (*self.slots.get())[handle.0].write(item);
            }
            Ok(handle)
        } else {
            Err(item)
        }
    }

    /// The returned handle must belong to this slab. TODO: ensure this statically?
    pub unsafe fn remove(&self, handle: SlabHandle) -> T {
        let item = (*self.slots.get())[handle.0].as_ptr().read();
        self.free_queue.enqueue(handle).unwrap();
        item
    }
}
