mod page_structs;

use self::page_structs::{PageDirectory, PageTable};
use core::ptr::Unique;
use crate::external_symbols::{kernel_start, kernel_end, get_ext_symb_add};
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
    let mut page_directory = PageDirectory(unsafe { Unique::new_unchecked(PAGE_DIR_ADDRESS as *mut _) });

    page_directory.clear();

    // Gdt, ps2 ports
    BITMAP.lock().kalloc_frame_by_address(0x0);
    page_directory.map_pages(0x0, 0x0);

    // VGA
    BITMAP.lock().kalloc_frame_by_address(0xb8000);
    page_directory.map_pages(0xb8000, 0xb8000);

    // Kernel mapping
    let kernel_first_page = get_ext_symb_add(kernel_start) & !0xFFF;
    let kernel_last_page = get_ext_symb_add(kernel_end) & !0xFFF;
    let mut i = kernel_first_page;
    while i <= kernel_last_page {
        BITMAP.lock().kalloc_frame_by_address(i);
        page_directory.map_pages(i, i);
        i += 0x1000;
    }

    // Recursive page directory trick
    page_directory.set_entry(1023, PAGE_DIR_ADDRESS, 0x1); 
    enable(PAGE_DIR_ADDRESS);
    *PAGE_DIRECTORY.lock() = unsafe { Some(PageDirectory(Unique::new_unchecked((0x3FFusize << 22 | 0x3FFusize << 12) as *mut _))) };
}

pub fn list_mappings() {
    if let Some(dir) = PAGE_DIRECTORY.lock().as_ref() {
        for i in 0..dir.ref_dir().len() - 1 {
            if dir.ref_dir()[i].is_present() {
                let table = unsafe { PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * i) as *mut _)) };
                for (j, entry) in table.ref_table().iter().enumerate().filter(|(_, e)| e.is_present()) {
                    println!("virtual: {:#010x} physical: {:#010x}", i << 22 | j << 12, entry.physical_memory_management_address());
                }
            }
        }
    }
}

use spin::Mutex;

pub static PAGE_DIRECTORY: Mutex<Option<PageDirectory>> = Mutex::new(None);
