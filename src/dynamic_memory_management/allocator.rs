use core::mem;
use crate::external_symbols::{get_kernel_end};
use crate::physical_memory_management::{BITMAP, PAGE_SIZE_4K};
use crate::virtual_memory_management::PAGE_DIRECTORY;

use core::ptr::{NonNull, Unique};
use core::fmt;
use core::alloc::{GlobalAlloc, Layout, Allocator, AllocError};
use super::{Locked};

/*
unsafe impl GlobalAlloc for Locked<KernelHeap> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().kalloc(layout).unwrap().as_ptr() as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.lock().free(ptr)
    }
}
*/

unsafe impl Allocator for Locked<KernelHeap> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            self.lock().kalloc(layout)
        }
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.lock().free(ptr)
    }
}

#[repr(C)]
#[derive(PartialEq)]
pub struct Chunk {
    pub size: usize,
    pub prev: *mut Chunk,
    pub next: *mut Chunk,
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "address: {:p} prev: {:p} next: {:p} size: {}",
               self, self.prev, self.next, self.size)
    }
}

unsafe impl Send for Chunk {}

pub struct KernelHeap {
    brk: Option<usize>,
    pub free_list: Option<*mut Chunk>,
}

unsafe impl Send for KernelHeap {}

impl KernelHeap {
    pub const fn new() -> KernelHeap {
        KernelHeap { 
            brk: None,
            free_list: None,
        }
    }

    fn find_block(&self, size: usize) -> Option<*mut Chunk> {
        if let Some(start) = self.free_list {
            unsafe {
                let mut head = start;

                // Do-while blackmagic
                while {
                    if (*head).size >= size {
                        return Some(head)
                    }
                    head = (*head).next;

                    head != start
                } {}
            }
        }
        None
    }

    pub fn kalloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>  {
        let required_space = layout.size() + mem::size_of::<Chunk>();
        let fit_chunk = self.find_block(required_space).ok_or(|| AllocError).or_else(|_| self.morecore(required_space))?;
        let new_chunk = (fit_chunk as usize + required_space) as *mut Chunk;

        unsafe {
            (*new_chunk).size = (*fit_chunk).size - required_space;
            (*fit_chunk).size = required_space;
            if (*fit_chunk).next == fit_chunk {
                (*new_chunk).next = new_chunk;
                (*new_chunk).prev = new_chunk;
            } else {
                KernelHeap::replace_chunk(fit_chunk, new_chunk);
            }
            self.free_list = Some(new_chunk);

            Ok(NonNull::new_unchecked(
                    core::slice::from_raw_parts_mut((fit_chunk as usize + 3 * mem::size_of::<usize>()) as *mut u8,
                    layout.size())))
        }
    }

    fn morecore(&mut self, required_space: usize) -> Result<*mut Chunk, AllocError> {
        let new_chunk = self.sbrk(required_space as isize)? as *mut Chunk;
        unsafe { 
            (*new_chunk).size = self.brk.unwrap() - new_chunk as usize; 
            self.free_in((new_chunk as usize + 3 * mem::size_of::<usize>()) as *mut u8);
        }
        self.free_list.ok_or(AllocError)
    }

    fn sbrk(&mut self, increment: isize) -> Result<usize, AllocError> {
        let old_brk = self.brk.unwrap_or({
            let new_brk = match get_kernel_end() {
                v if v & 0xFFF == 0 => v,
                v => (v & !0xFFF) + PAGE_SIZE_4K,
            };
            self.brk = Some(new_brk);
            new_brk
        });

        let is_neg = increment < 0;
        let mut required_pages = (increment / PAGE_SIZE_4K as isize).abs() as usize
            + (increment % PAGE_SIZE_4K as isize != 0) as usize;
        while required_pages > 0 {
            let current_brk = self.brk.unwrap();
            if is_neg {
                let new_brk = current_brk - PAGE_SIZE_4K;
                PAGE_DIRECTORY.lock().unmap_pages(new_brk);
                self.brk = Some(new_brk);
            } else {
                let p_add = BITMAP.lock().kalloc_frame().map_err(|_| AllocError)?;
                PAGE_DIRECTORY.lock().map_pages(p_add, current_brk).map_err(|_| AllocError)?;
                self.brk = Some(current_brk + PAGE_SIZE_4K);
            }
            required_pages -= 1;
        }
        Ok(old_brk)
    }
    
    pub unsafe fn free(&mut self, address: NonNull<u8>) {
        self.free_in(address.as_ptr())
    }

    unsafe fn free_in(&mut self, address: *mut u8) {
        let mut new_chunk = ((address as usize - 3 * mem::size_of::<usize>()) as *mut Chunk);
        self.free_list = Some(match self.free_list {
            None => {
                (*new_chunk).next = new_chunk;
                (*new_chunk).prev = new_chunk;
                new_chunk
            },
            Some(start) if (*start).next == start => {
                if (start as usize + (*start).size) as *mut Chunk == new_chunk {
                    (*start).size += (*new_chunk).size;
                    start
                } else if (new_chunk as usize + (*new_chunk).size) as *mut Chunk == start {
                    (*new_chunk).size += (*start).size;
                    (*new_chunk).next = new_chunk;
                    (*new_chunk).prev = new_chunk;
                    new_chunk
                } else {
                    (*start).next = new_chunk;
                    (*start).prev = new_chunk;
                    (*new_chunk).next = start;
                    (*new_chunk).prev = start;
                    new_chunk
                }
            },
            Some(start) => {
                let mut head = start;
                while !(head < new_chunk && new_chunk < (*head).next)
                    && !(head > (*head).next && (new_chunk > head || new_chunk < (*head).next)) {
                        head = (*head).next;
                    }
                KernelHeap::insert_chunk(head, new_chunk)
            },
        }); 
    }

    unsafe fn replace_chunk(head: *mut Chunk, new: *mut Chunk) {
        (*new).next = (*head).next;
        (*new).prev = (*head).prev;
        (*(*new).next).prev = new;
        (*(*new).prev).next = new;
    }

    unsafe fn insert_chunk(head: *mut Chunk, new: *mut Chunk) -> *mut Chunk {
        if (new as usize + (*new).size) as *mut Chunk == (*head).next {
            (*new).size += (*(*head).next).size;
            (*new).next = (*(*head).next).next;
            (*(*new).next).prev = new;
        } else {
            (*new).next = (*head).next;
            (*(*new).next).prev = new;
        }

        if (head as usize + (*head).size) as *mut Chunk == new {
            (*head).size += (*new).size;
            (*head).next = (*new).next;
            (*(*head).next).prev = head;
            return head;
        } else {
            (*new).prev = head;
            (*head).next = new;
        }
        return new;
    }

    unsafe fn remove_chunk(head: *mut Chunk, to_del: *mut Chunk) {
        (*head).next = (*to_del).next;
        (*(*head).next).prev = head;
    }
}

impl fmt::Display for KernelHeap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(start) = self.free_list {
            let mut head = start;

            // Do-while blackmagic
            while {
                unsafe {
                    if let Err(e) = write!(f, "{}\n", head.as_ref().unwrap()) {
                        return Err(e);
                    }
                    head = (*head).next;
                }

                head != start
            } {}
        }
        Ok(())
    }
}

use spin::Mutex;

pub static KERNEL_HEAP: Locked<KernelHeap> = Locked::new(KernelHeap::new());
