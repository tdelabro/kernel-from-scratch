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

pub struct Paging(bool);

impl Paging {
    pub fn is_enabled(&self) -> bool {
        self.0
    }

    fn activate(&mut self) {
        self.0 = true;
    }

    fn deactivate(&mut self) {
        self.0 = false;
    }

    pub unsafe fn get_dir(&self) -> PageDirectory {
        assert!(self.is_enabled(), "paging is not enabled");
        PageDirectory(Unique::new_unchecked((0x3FFusize << 22 | 0x3FFusize << 12) as *mut _))
    }

    pub unsafe fn get_table(&self, index: usize) -> Option<PageTable> {
        assert!(self.is_enabled(), "paging is not enabled");
        let dir = self.get_dir();
        if !dir.ref_dir()[index].is_present() {
            return None;  
        }
        Some(PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * index) as *mut _)))
    }

    pub fn list_mappings(&self) {
        assert!(self.is_enabled(), "paging is not enabled");
        let dir = unsafe { self.get_dir() };
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

    pub fn init(&mut self) {
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
        self.activate();
    }

}

use spin::Mutex;

pub static PAGING: Mutex<Paging> = Mutex::new(Paging(false));

