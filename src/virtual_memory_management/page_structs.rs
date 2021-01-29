use core::ptr::Unique;
use core::fmt;

pub struct PageTableEntry(usize);

impl PageTableEntry {
    fn new(address: usize, flags: usize) -> PageTableEntry {
        assert_eq!(0, address & 0xFFF);
        PageTableEntry(address | flags)
    }

    pub fn page_frame_address(&self) -> usize {
        (self.0 & 0xFFFFF000) as usize
    }

    pub fn is_present(&self) -> bool {
        self.0 & 0x1 != 0
    }

    fn is_wr(&self) -> bool {
        self.0 & (0x1 << 1) != 0
    }
}

impl fmt::Display for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Frame physical address: {:#010x}, Present: {}, Write/Read: {}",
               self.page_frame_address(), self.is_present(), self.is_wr())
    }
}

pub struct PageTable(pub Unique<[PageTableEntry; 1024]>);

impl PageTable {
    pub fn ref_table(&self) -> &[PageTableEntry; 1024] {
        unsafe { self.0.as_ref() }
    }

    fn mut_table(&mut self) -> &mut [PageTableEntry; 1024] {
        unsafe { self.0.as_mut() }
    }

    fn clear(&mut self) {
        for i in 0..self.ref_table().len() {
            self.mut_table()[i] = PageTableEntry::new(0, 0)
        }
    }
}

impl fmt::Display for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, entry) in self.ref_table().iter().enumerate().filter(|(_, e)| e.is_present()) {
            if let Err(e) = write!(f, "{:04}: {}\n", idx, entry) {
                return Err(e)
            }
        }
        Ok(())
    }
}

pub struct PageDirectoryEntry(usize);

impl PageDirectoryEntry {
    fn new(address: usize, flags: usize) -> PageDirectoryEntry {
        assert_eq!(0, address & 0xFFF);
        PageDirectoryEntry(address | flags)
    }

    pub fn page_table_address(&self) -> usize {
        (self.0 & 0xFFFFF000) as usize
    }

    pub fn is_present(&self) -> bool {
        self.0 & 0x1 != 0
    }

    fn is_wr(&self) -> bool {
        self.0 & (0x1 << 1) != 0
    }
}

impl fmt::Display for PageDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Table physical address: {:#010x}, Present: {}, Write/Read: {}",
               self.page_table_address(), self.is_present(), self.is_wr())
    }
}

use crate::physical_memory_management::BITMAP;

pub struct PageDirectory(pub Unique<[PageDirectoryEntry; 1024]>, pub bool);

impl PageDirectory {
    pub fn enabled(&self) -> bool {
        self.1
    }

    pub fn ref_dir(&self) -> &[PageDirectoryEntry; 1024] {
        unsafe { self.0.as_ref() }
    }

    fn mut_dir(&mut self) -> &mut [PageDirectoryEntry; 1024] {
        unsafe { self.0.as_mut() }
    }

    pub fn clear(&mut self) {
        for i in 0..self.ref_dir().len() {
            self.mut_dir()[i] = PageDirectoryEntry::new(0, 0)
        }
    }

    pub fn map_pages(&mut self, physical_page_address: usize, virtual_page_address: usize) {
        if self.enabled() {
            self.map_pages_e(physical_page_address, virtual_page_address);
        } else {
            self.map_pages_d(physical_page_address, virtual_page_address);
        }
    }

    fn map_pages_d(&mut self, physical_page_address: usize, virtual_page_address: usize) {
        assert_eq!(0, physical_page_address & 0xFFF, "physical address is not aligned: {:#10x}", physical_page_address);
        assert_eq!(0, virtual_page_address & 0xFFF, "physical address is not aligned: {:#10x}", virtual_page_address);

        let d_offset = virtual_page_address >> 22;
        let t_offset = (virtual_page_address & 0x3FF000) >> 12;

        let mut page_table: PageTable;
        let page_table_add: usize;
        
        if !self.ref_dir()[d_offset].is_present() {
            page_table_add = BITMAP.lock().kalloc_frame();
            page_table = unsafe { PageTable(Unique::new_unchecked(page_table_add as *mut _)) };
            page_table.clear();
            self.mut_dir()[d_offset] = PageDirectoryEntry::new(page_table_add, 0x1);
        } else {
            page_table_add = self.ref_dir()[d_offset].page_table_address();
            page_table = unsafe { PageTable(Unique::new_unchecked(page_table_add as *mut _)) };
        }
        page_table.mut_table()[t_offset] = PageTableEntry::new(physical_page_address, 0x1);
    }

    fn map_pages_e(&mut self, physical_page_address: usize, virtual_page_address: usize) {
        assert_eq!(0, physical_page_address & 0xFFF, "physical address is not aligned: {:#10x}", physical_page_address);
        assert_eq!(0, virtual_page_address & 0xFFF, "physical address is not aligned: {:#10x}", virtual_page_address);

        let d_offset = virtual_page_address >> 22;
        let t_offset = (virtual_page_address & 0x3FF000) >> 12;

        let mut page_table: PageTable;
        let page_table_add: usize;

        if !self.ref_dir()[d_offset].is_present() {
            page_table_add = BITMAP.lock().kalloc_frame();
            self.mut_dir()[d_offset] = PageDirectoryEntry::new(page_table_add, 0x1);
            page_table = unsafe { PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * d_offset) as *mut _)) };
            page_table.clear();
        } else {
            page_table = unsafe { PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * d_offset) as *mut _)) };
        }

        page_table.mut_table()[t_offset] = PageTableEntry::new(physical_page_address, 0x1);
    }

    pub fn set_entry(&mut self, index: usize, address: usize, tags: usize) {
        self.mut_dir()[index] = PageDirectoryEntry::new(address, tags);
    }

    pub fn get_available_page_address_in_range(&self, min: usize, max: usize) -> Option<usize> {
        for i in min >> 22 .. self.ref_dir().len() - 1 {
            let table = unsafe { PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * i) as *mut _)) };
            for j in 0..table.ref_table().len() {
                let page_add = i << 22 | j << 12;
                if page_add >= max {
                    return None
                }
                if !table.ref_table()[j].is_present() {
                    return Some(page_add)
                }
            }
        }
        None
    }

    pub fn list_mappings(&self) {
        assert!(self.enabled());
        for i in 0..self.ref_dir().len() - 1 {
            if self.ref_dir()[i].is_present() {
                let table = unsafe { PageTable(Unique::new_unchecked((0xFFC00000usize + 4 * i) as *mut _)) };
                for (j, entry) in table.ref_table().iter().enumerate().filter(|(_, e)| e.is_present()) {
                    println!("v: {:#010x} p: {:#010x}", i << 22 | j << 12, entry.page_frame_address());
                }
            }
        }
    }

}

impl fmt::Display for PageDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, entry) in self.ref_dir().iter().enumerate().filter(|(_, e)| e.is_present()) {
            if let Err(e) = write!(f, "{:04}: {}\n", idx, entry) {
                return Err(e)
            }
        }
        Ok(())
    }
}
