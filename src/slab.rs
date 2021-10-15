// Preallocated storage similar to slab crate, but fixed size

use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// Handle to a queued item
#[derive(Debug)]
pub struct SlabHandle {
    index: usize,
    // Pointer of the Arc<Slab> used to pin to specific instance
    ptr: usize,
}

// N should be a power of 2 so that overflows work correctly, but 64bit usize is unlikely to ever overflow
pub struct Slab<T, const N: usize> {
    // SPMC ring buffer. Contains indices of free slots or usize::MAX if empty.
    // SlabReceiver is producer in this case, because it pushes returned indices.
    free_queue: [AtomicUsize; N],
    // Number of used free_queue slots. Zero means that Slab is empty, N means that Slab is full.
    free_used: AtomicUsize,
    // Current pop location of free queue
    free_queue_tail: AtomicUsize,
    // Slots that store actual data
    slots: UnsafeCell<[MaybeUninit<T>; N]>,
}

unsafe impl<T: Send, const N: usize> Send for Slab<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for Slab<T, N> {}

impl<T, const N: usize> Slab<T, N> {
    pub fn new() -> Self {
        let free_queue = unsafe {
            let mut buf = MaybeUninit::uninit();
            let buf_ptr = buf.as_mut_ptr() as *mut AtomicUsize;
            (0..N).for_each(|i| buf_ptr.add(i).write(AtomicUsize::new(i)));
            buf.assume_init()
        };

        // Need unsafe, otherwise compiler complains about T not being copy
        let slots = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        Self {
            free_queue,
            free_used: AtomicUsize::new(0),
            free_queue_tail: AtomicUsize::new(0),
            slots: UnsafeCell::new(slots),
        }
    }

    pub fn split(self) -> (SlabSender<T, N>, SlabReceiver<T, N>) {
        let inner = Arc::new(self);

        (
            SlabSender {
                inner: inner.clone(),
            },
            SlabReceiver {
                inner,
                free_queue_head: 0,
            },
        )
    }
}

#[derive(Clone)]
pub struct SlabSender<T, const N: usize> {
    inner: Arc<Slab<T, N>>,
}

impl<T, const N: usize> SlabSender<T, N> {
    pub fn insert(&self, item: T) -> Result<SlabHandle, T> {
        let index = match self.get_index() {
            Some(index) => index,
            None => return Err(item),
        };

        unsafe {
            (*self.inner.slots.get())[index].write(item);
        }

        Ok(SlabHandle {
            index,
            ptr: Arc::as_ptr(&self.inner) as usize,
        })
    }

    fn get_index(&self) -> Option<usize> {
        let free_used = self.inner.free_used.fetch_add(1, Ordering::Acquire);
        if free_used >= N {
            self.inner.free_used.fetch_sub(1, Ordering::Release);
            return None;
        }

        let tail = self.inner.free_queue_tail.fetch_add(1, Ordering::Acquire);
        let val = self.inner.free_queue[tail % N].swap(usize::MAX, Ordering::Release);
        assert!(val != usize::MAX);
        Some(val)
    }
}

pub struct SlabReceiver<T, const N: usize> {
    inner: Arc<Slab<T, N>>,
    // push location of free_queue
    free_queue_head: usize,
}

impl<T, const N: usize> SlabReceiver<T, N> {
    pub fn remove(&mut self, handle: SlabHandle) -> T {
        // Ensure handle belongs to this slab
        assert!(Arc::as_ptr(&self.inner) as usize == handle.ptr);

        let item = unsafe { (*self.inner.slots.get())[handle.index].as_ptr().read() };
        self.return_index(handle.index);
        item
    }

    fn return_index(&mut self, index: usize) {
        let old = self.inner.free_queue[self.free_queue_head % N].swap(index, Ordering::Acquire);
        assert!(old == usize::MAX);

        let count = self.inner.free_used.fetch_sub(1, Ordering::Release);
        assert!(count != usize::MAX);

        self.free_queue_head += 1;
    }
}
