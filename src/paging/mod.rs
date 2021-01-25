mod page_structs;

use self::page_structs::{PageDirectory, PageTable};

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

use core::ptr::Unique;
use crate::external_symbols::{kernel_start, kernel_end, get_ext_symb_add};

pub fn init() {

    let mut page_directory = PageDirectory(unsafe { Unique::new_unchecked(PAGE_DIR_ADDRESS as *mut _) });

    page_directory.clear();
    
    // Gdt
    page_directory.map_pages(crate::gdt::GDTBASE & !0xFFF, crate::gdt::GDTBASE & !0xFFF);

    // VGA
    page_directory.map_pages(0xb8000, 0xb8000);
    
    // Kernel mapping
    page_directory.map_range_pages(
        get_ext_symb_add(kernel_start),
        get_ext_symb_add(kernel_end),
        get_ext_symb_add(kernel_start),
    );

    // Recursive page directory trick
    page_directory.set_entry(1023, PAGE_DIR_ADDRESS, 0x1); 
    enable(PAGE_DIR_ADDRESS);
}

pub unsafe fn get_dir() -> PageDirectory {
    PageDirectory(Unique::new_unchecked((0x3FFusize << 22 | 0x3FFusize << 12) as *mut _))
}


pub fn list_mappings() {
    let dir = unsafe { get_dir() };
    for i in 0..dir.ref_dir().len() - 1 {
        if dir.ref_dir()[i].is_present() {
            let table = unsafe { PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * i) as *mut _)) };
            for j in 0..1024 {
                if table.ref_table()[j].is_present() {
                    println!("virtual: {:#010x} physical: {:#010x}", i << 22 | j << 12, table.ref_table()[j].page_frame_address());
                }
            }
        }
    }
}
