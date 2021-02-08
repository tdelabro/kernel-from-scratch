use core::ptr::Unique;
use super::{Locked};
use super::allocator::{KernelHeap, KERNEL_HEAP};
use core::fmt;
use core::alloc::{Layout, Allocator};
use core::ptr::NonNull;
use core::alloc::GlobalAlloc;

pub struct Box<T: ?Sized, A: Allocator = &'static Locked<KernelHeap>>(Unique<T>, A);

use core::ops::{Drop, Deref, DerefMut};

impl<T: ?Sized, A: Allocator> Deref for Box<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized, A: Allocator> DerefMut for Box<T, A> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<T: ?Sized, A: Allocator> Drop for Box<T, A> {
    fn drop(&mut self) {
        unsafe {
            let p = NonNull::new_unchecked(self.deref_mut() as *mut T as *mut u8);
            self.1.deallocate(p, Layout::for_value(self));
        }
    }
}

impl<T: ?Sized, A: Allocator> fmt::Pointer for Box<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.deref(), f)
    }
}

impl<T: fmt::Display + ?Sized, A: Allocator> fmt::Display for Box<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}

impl<T, A: Allocator> Box<T, A> {
    pub fn new_in(value: T, allocator: A) -> Box<T, A> {
        let p = unsafe {
            Unique::new_unchecked(allocator.allocate(Layout::new::<T>()).unwrap().as_ptr() as *mut T)
        };
        let mut b = Box(p, allocator);
        *b = value;
        b
    }

}

impl<T> Box<T> {
    pub fn new(value: T) -> Box<T, &'static Locked<KernelHeap>> {
        let p = unsafe {
            Unique::new_unchecked(KERNEL_HEAP.alloc(Layout::new::<T>()) as *mut T)
        };
        let mut b = Box(p, &KERNEL_HEAP);
        *b = value;
        b
    }
}
