use core::cell::Cell;
use core::ptr::NonNull;
use core::marker::PhantomData;

pub struct Rc<T: ?Sized> {
    ptr: NonNull<RcBox<T>>,
    phantom: PhantomData<RcBox<T>>,
}

struct RcBox<T: ?Sized> {
    strong: Cell<usize>,
    refcount: Cell<usize>,
    value: T,
}

impl<T: ?Sized> Clone for Rc<T> {
    fn clone(&self) -> Rc<T> {
        self.inc_strong();
        Rc {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

trait RcBoxPtr<T: ?Sized> {
    fn inner(&self) -> &RcBox<T>;

    fn strong(&self) -> usize {
        self.inner().strong.get()
    }

    fn inc_strong(&self) {
        self.inner()
            .strong
            .set(self.strong()
                     .checked_add(1)
                     .unwrap());
    }
}

impl<T: ?Sized> RcBoxPtr<T> for Rc<T> {
   fn inner(&self) -> &RcBox<T> {
       unsafe {
           self.ptr.as_ref()
       }
   }
}

