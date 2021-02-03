mod page_structs;

use self::page_structs::{PageDirectory, PageTable};
use core::ptr::Unique;
use crate::external_symbols::{get_kernel_start, get_kernel_end};
use crate::physical_memory_management::BITMAP;

const PAGE_DIR_ADDRESS: usize = 0x21000;

fn enable(page_dir_address: usize) {
    unsafe {
        asm!("mov cr3, eax
            mov ebx, cr0
            or ebx, 0x80000000
            mov cr0, ebx",
            in("eax") page_dir_address, out("ebx") _,
            options(nostack));
    }
}

pub fn init() {
    PAGE_DIRECTORY.lock().clear();

    // Gdt, ps2 ports
    BITMAP.lock().kalloc_frame_by_address(0x0);
    PAGE_DIRECTORY.lock().map_pages(0x0, 0x0);

    // VGA
    BITMAP.lock().kalloc_frame_by_address(0xb8000);
    PAGE_DIRECTORY.lock().map_pages(0xb8000, 0xb8000);

    // Kernel mapping
    let kernel_first_page = get_kernel_start() & !0xFFF;
    let kernel_last_page = get_kernel_end() & !0xFFF;
    let mut i = kernel_first_page;
    while i <= kernel_last_page {
        BITMAP.lock().kalloc_frame_by_address(i);
        PAGE_DIRECTORY.lock().map_pages(i, i);
        i += 0x1000;
    }

    // Recursive page directory trick
    PAGE_DIRECTORY.lock().set_entry(1023, PAGE_DIR_ADDRESS, 0x1); 
    enable(PAGE_DIR_ADDRESS);
    *PAGE_DIRECTORY.lock() = unsafe { PageDirectory(Unique::new_unchecked((0x3FFusize << 22 | 0x3FFusize << 12) as *mut _), true) };
}

use spin::Mutex;

pub static PAGE_DIRECTORY: Mutex<PageDirectory> = Mutex::new(PageDirectory(
        unsafe { Unique::new_unchecked(PAGE_DIR_ADDRESS as *mut _) },
        false));
