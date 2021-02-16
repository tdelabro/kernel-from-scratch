use core::mem;
use crate::external_symbols::{get_first_page_after_kernel};
use crate::physical_memory_management::{BITMAP, PAGE_SIZE_4K};
use crate::virtual_memory_management::PAGE_DIRECTORY;

use core::ptr::{NonNull};
use alloc::alloc::{GlobalAlloc, Layout, Allocator, AllocError};
use super::{Locked};

unsafe impl GlobalAlloc for Locked<Heap> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate(layout).unwrap().cast::<u8>().as_ptr()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate(NonNull::new(ptr).unwrap(), layout);
    }
}

unsafe impl Allocator for Locked<Heap> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.lock().malloc(layout)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _: Layout) {
        self.lock().free(ptr);
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

pub struct Heap {
    start: *const usize,
    brk: *const usize,
    free_list: Option<*mut Chunk>,
    is_supervisor: bool,
}

unsafe impl Send for Heap {}

impl Heap {
    pub const unsafe fn new(start_add: *const usize, is_supervisor: bool) -> Heap {
        Heap { 
            start: start_add,
            brk: start_add,
            free_list: None,
            is_supervisor: is_supervisor,
        }
    }

    fn get_brk(&self) -> usize {
        self.brk as usize
    }

    fn set_brk(&mut self, new_brk: usize) {
        self.brk = new_brk as *const usize;
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

    fn malloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>  {
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
                Heap::replace_chunk(fit_chunk, new_chunk);
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
            (*new_chunk).size = self.get_brk() - new_chunk as usize; 
            self.free_in((new_chunk as usize + 3 * mem::size_of::<usize>()) as *mut u8);
        }
        self.free_list.ok_or(AllocError)
    }

    fn sbrk(&mut self, increment: isize) -> Result<usize, AllocError> {
        let old_brk = self.get_brk();
        let is_neg = increment < 0;
        let mut required_pages = (increment / PAGE_SIZE_4K as isize).abs() as usize
            + (increment % PAGE_SIZE_4K as isize != 0) as usize;
            while required_pages > 0 {
                let current_brk = self.get_brk();
                if is_neg {
                    let new_brk = current_brk - PAGE_SIZE_4K;
                    if PAGE_DIRECTORY.lock().is_enabled() {
                        PAGE_DIRECTORY.lock().unmap_pages(new_brk)
                            .map_err(|_| AllocError)?;
                    } else {
                        BITMAP.lock().free_frame(new_brk).map_err(|_| AllocError)?;
                    }
                    self.set_brk(new_brk);
                } else {
                    if PAGE_DIRECTORY.lock().is_enabled() {
                        let p_add = BITMAP.lock().alloc_frame().map_err(|_| AllocError)?;
                        PAGE_DIRECTORY
                            .lock()
                            .map_pages(p_add, current_brk, if self.is_supervisor { 0x3 } else { 0x7 })
                            .map_err(|_| AllocError)?;
                    } else {
                        BITMAP.lock().alloc_frame_by_address(current_brk).map_err(|_| AllocError)?;
                    }
                    self.set_brk(current_brk + PAGE_SIZE_4K);
                }
                required_pages -= 1;
            }
        Ok(old_brk)
    }

    unsafe fn free(&mut self, address: NonNull<u8>) {
        self.free_in(address.as_ptr());
        self.release_memory();
    }

    unsafe fn free_in(&mut self, address: *mut u8) {
        let mut new_chunk = (address as usize - 3 * mem::size_of::<usize>()) as *mut Chunk;
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
                Heap::insert_chunk(head, new_chunk)
            },
        }); 
    }

    fn release_memory(&mut self) {
        if let Some(node) = self.free_list {
            unsafe {
                if (*node).size > PAGE_SIZE_4K && (*node).next <= node {
                    self.sbrk(PAGE_SIZE_4K as isize * -1).unwrap();
                    (*self.free_list.unwrap()).size -= PAGE_SIZE_4K;
                }
            }
        }
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

    pub unsafe fn size(address: NonNull<u8>) -> usize {
        *address.cast::<usize>().as_ptr().offset(-3)
    }
}

use core::fmt;

impl fmt::Display for Heap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Heap end at: {:p} and accessible by {}.\n", self.brk,
               if self.is_supervisor { "supervisor only" } else {"everybody"})?;
        if let Some(start) = self.free_list {
            let mut head = start;
            write!(f, "Free chunks list:\n")?;

            // Do-while blackmagic
            while {
                unsafe {
                    write!(f, "{}\n", head.as_ref().unwrap())?;
                    head = (*head).next;
                }

                head != start
            } {}
        } else {
            write!(f, "No free chunks\n")?;
        }
        Ok(())
    }
}

use core::ops::Drop;

impl Drop for Heap {
    fn drop(&mut self) {
        while self.brk > self.start {
          self.sbrk(PAGE_SIZE_4K as isize * -1).unwrap();
        }
    }
}

#[global_allocator]
pub static KERNEL_HEAP: Locked<Heap> = Locked::new(unsafe {
    Heap::new(get_first_page_after_kernel(), true)
});
