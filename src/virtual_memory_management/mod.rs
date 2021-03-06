//! Paging management
//!
//! Keep track of an unique page directory. Dynamicaly manage page tables.

mod page_structs;

use self::page_structs::PageDirectory;
use crate::external_symbols::{get_kernel_end, get_kernel_start};
use crate::physical_memory_management::{BITMAP, PAGE_SIZE_4K};
use crate::MultibootInfo;
use core::convert::TryInto;
use core::ptr::Unique;

/// Physical address of the page directory frame
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

/// Setup memory
///
/// If arg is false, will tag specific area of memory as used in BITMAP, to avoid latter override.
/// Else if arg is true it will also identity page those memory area, set up the recursive page directory trick on the last entry
/// of the page directory and enable paging.
///
/// Target memory areas:
/// - Global Descriptor Table
/// - Ps2 ports
/// - VGA screen memory map
/// - The whole kernel
pub fn init(enable_paging: bool, multiboot_info: MultibootInfo) {
    // Only let available the RAM really provided by the system
    let mem_map = multiboot_info.get_memory_map().unwrap();
    unsafe {
        assert_eq!(
            (*mem_map.inner).entry_version,
            0,
            "multiboot memory doesn't use version 0 entries"
        );
        for mem_entry in mem_map {
            if (*mem_entry).typ != 1 {
                let mut c_mem = 0;
                let base_frame = (*mem_entry).base_addr & !0xFFF;
                let end = (*mem_entry).base_addr + (*mem_entry).length;

                while base_frame + c_mem < end {
                    BITMAP
                        .lock()
                        .alloc_frame_by_address((base_frame + c_mem).try_into().unwrap())
                        .unwrap();
                    c_mem += PAGE_SIZE_4K as u64;
                }
            }
        }
    }

    let kernel_first_page = get_kernel_start() as usize & !0xFFF;
    let kernel_last_page = get_kernel_end() as usize & !0xFFF;
    let multiboot_frame_add = multiboot_info.inner as usize & !0xFFF;

    if enable_paging {
        PAGE_DIRECTORY.lock().clear();

        // Gdt, ps2 ports
        BITMAP.lock().alloc_frame_by_address(0x0).unwrap();
        PAGE_DIRECTORY.lock().map_pages(0x0, 0x0, 0x3).unwrap();

        // VGA
        BITMAP.lock().alloc_frame_by_address(0xb8000).unwrap();
        PAGE_DIRECTORY
            .lock()
            .map_pages(0xb8000, 0xb8000, 0x3)
            .unwrap();

        // Kernel mapping
        let mut i = kernel_first_page;
        while i <= kernel_last_page {
            BITMAP.lock().alloc_frame_by_address(i).unwrap();
            PAGE_DIRECTORY.lock().map_pages(i, i, 0x3).unwrap();
            i += 0x1000;
        }

        // MultibootInfo
        if BITMAP
            .lock()
            .alloc_frame_by_address(multiboot_frame_add)
            .is_ok()
        {
            PAGE_DIRECTORY
                .lock()
                .map_pages(multiboot_frame_add, multiboot_frame_add, 0x3)
                .unwrap();
        }

        // Recursive page directory trick
        PAGE_DIRECTORY.lock().set_entry(1023, PAGE_DIR_ADDRESS, 0x3);
        enable(PAGE_DIR_ADDRESS);
        *PAGE_DIRECTORY.lock() = unsafe {
            PageDirectory(
                Unique::new_unchecked((0x3FFusize << 22 | 0x3FFusize << 12) as *mut _),
                true,
            )
        };
    } else {
        BITMAP.lock().alloc_frame_by_address(0x0).unwrap();
        BITMAP.lock().alloc_frame_by_address(0xb8000).unwrap();
        let mut i = kernel_first_page;
        while i <= kernel_last_page {
            BITMAP.lock().alloc_frame_by_address(i).unwrap();
            i += 0x1000;
        }
        BITMAP
            .lock()
            .alloc_frame_by_address(multiboot_frame_add)
            .ok();
    }
}

use spin::Mutex;

/// Unique access to the page directory
pub static PAGE_DIRECTORY: Mutex<PageDirectory> = Mutex::new(PageDirectory(
    unsafe { Unique::new_unchecked(PAGE_DIR_ADDRESS as *mut _) },
    false,
));
