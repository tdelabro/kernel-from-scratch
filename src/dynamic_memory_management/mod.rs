use crate::external_symbols::{kernel_end, get_ext_symb_add};


#[repr(C)]
struct Header {
    next: &mut Header,
    size: usize,
}

struct KernelHeap {
    free_list: &mut Header,
}

pub fn kmalloc(size: usize) -> *mut _ {
    if let None = KERNEL_HEAP.lock() {
        // get new space
    }

}

pub fn kfree(address: *mut _) {

}

static KERNEL_HEAP: Mutex<Option<KernelHeap>> = Mutex::new(None);
