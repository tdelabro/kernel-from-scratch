use core::mem;
use crate::external_symbols::{kernel_end, get_ext_symb_add};
use crate::physical_memory_management::BITMAP;
use crate::virtual_memory_management::PAGE_DIRECTORY;

use core::ptr::Unique;
pub struct Box<T>(Unique<T>);

use core::ops::{Drop, Deref, DerefMut};

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        KERNEL_HEAP.lock().kfree(self);
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

#[repr(C)]
#[derive(PartialEq)]
pub struct Node {
    pub next: *mut Node,
    pub size: usize,
}

unsafe impl Send for Node {}

pub struct KernelHeap {
    pub free_list: Option<*mut Node>,
}

unsafe impl Send for KernelHeap {}

impl KernelHeap {
    pub const fn new() -> KernelHeap {
        KernelHeap { free_list: None }
    }

    fn find_block(&self, size: usize) -> Option<(*mut Node, *mut Node)> {
        if let Some(start) = self.free_list {
            unsafe {
                let mut prev = start;
                let mut head = (*start).next;

                // Do-while blackmagic
                while {
                    if (*head).size >= size {
                        return Some((prev, head))
                    }
                    prev = head;
                    head = (*head).next;

                    head != start
                } {}
            }
        }
        None
    }

    pub fn kalloc<'a, T>(&mut self) -> Box<T> {
        let size = mem::size_of::<T>();
        let mut block = self.find_block(size);
        if block.is_none() {
            self.morecore(size);
            block = self.find_block(size);
        }
        let (prev, header) = block.unwrap();
        let next_header = (header as usize + mem::size_of::<Node>() + size) as *mut Node;
        unsafe {
            (*next_header).size = (*header).size - mem::size_of::<Node>() - size;
            (*header).size = size;

            if prev == header {
                (*next_header).next = next_header;
            } else {
                (*next_header).next = (*header).next;
                (*prev).next = next_header;
            }
            self.free_list = Some(next_header);
            
            Box(Unique::new((header.offset(1) as *mut T)).unwrap())
        }
    }

    fn morecore(&mut self, size: usize) {
        let v_add = PAGE_DIRECTORY.lock().get_available_page_address_in_range(0, 0x3FFFFFFF).unwrap();
        let p_add = BITMAP.lock().kalloc_frame();
        let header: *mut Node;
        PAGE_DIRECTORY.lock().map_pages(p_add, v_add);
        header = v_add as *mut Node;
        unsafe {
            (*header).size = 4096 - mem::size_of::<Node>();
            self.kfree_in(header.offset(1) as *mut u8);
        }
    }

    pub fn kfree<T>(&mut self, address: &mut Box<T>) {
        unsafe { self.kfree_in(address.0.as_ptr() as *mut T as *mut u8) };
    }

    unsafe fn kfree_in(&mut self, address: *mut u8) {
        let header = (address as *mut Node).offset(-1);

        if let Some(node) = self.free_list {
            let mut head = node;
            while ((*head).next < header) {
                head = (*head).next;
            }
            (*header).next = (*head).next;
            (*head).next = header;
            self.free_list = Some(header)
        } else {
            (*header).next = header;
            self.free_list = Some(header);
        }
    }
}

use core::fmt;

impl fmt::Display for KernelHeap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(start) = self.free_list {
            let mut head = start;

            // Do-while blackmagic
            while {
                unsafe {
                    if let Err(e) =  write!(f, "address: {:p} size: {} next: {:p}\n", head, (*head).size, (*head).next) {
                        return Err(e);
                    }
                    head =  (*head).next;
                }

                head != start
            } {}
        }
        Ok(())
    }
}

use spin::Mutex;

pub static KERNEL_HEAP: Mutex<KernelHeap> = Mutex::new(KernelHeap::new());
