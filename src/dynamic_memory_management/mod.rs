//! A simple heap
//!
//! Implement the Allocator and GlobalAlloc traits so it can be used with rust native smart
//! pointers.

mod allocator;

pub use self::allocator::{Heap, KERNEL_HEAP};

/// An immutable wrapper around Mutex
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}
