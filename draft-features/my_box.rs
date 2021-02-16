use core::ptr::Unique;
use super::{Locked};
use super::allocator::{Heap, KERNEL_HEAP};
use core::fmt;
use core::alloc::{Layout, Allocator};
use core::ptr::NonNull;
use core::alloc::GlobalAlloc;
use core::convert::{AsMut, AsRef};

pub struct Box<T: ?Sized, A: Allocator = &'static Locked<Heap>>(Unique<T>, A);

use core::ops::{Drop, Deref, DerefMut};

impl<T: ?Sized, A: Allocator> Deref for Box<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized, A: Allocator> DerefMut for Box<T, A> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T: ?Sized, A: Allocator> Drop for Box<T, A> {
    fn drop(&mut self) {
        unsafe {
            let p = NonNull::new_unchecked(self.as_mut()).cast::<u8>();
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

impl<T: fmt::Debug + ?Sized, A: Allocator> fmt::Debug for Box<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<T: ?Sized, A: Allocator> AsMut<T> for Box<T, A> {
    fn as_mut(&mut self) -> &mut T {
       unsafe { self.0.as_mut() }
    }
}

impl<T: ?Sized, A: Allocator> AsRef<T> for Box<T, A> {
    fn as_ref(&self) -> &T {
       unsafe { self.0.as_ref() }
    }
}

impl<T, A: Allocator> Box<T, A> {
    pub fn new_in(value: T, allocator: A) -> Self {
        let p = unsafe {
            Unique::new_unchecked(allocator.allocate(Layout::new::<T>()).unwrap().as_ptr() as *mut T)
        };
        let mut b = Box(p, allocator);
        *b = value;
        b
    }

}

impl<T: Sized> Box<T> {
    pub fn new(value: T) -> Box<T, &'static Locked<Heap>> {
        let p = unsafe {
            Unique::new_unchecked(KERNEL_HEAP.alloc(Layout::new::<T>()) as *mut T)
        };
        let mut b = Box(p, &KERNEL_HEAP);
        *b = value;
        b
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}
