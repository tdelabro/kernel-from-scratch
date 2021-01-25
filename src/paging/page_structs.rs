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
        write!(f, "Address: {:#010x}, Present: {}, Write/Read: {}", self.page_frame_address(), self.is_present(), self.is_wr())
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
        for entry in self.ref_table() {
            if entry.is_present() {
                if let Err(e) = write!(f, "{}\n", entry) {
                    return Err(e)
                }
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
        write!(f, "Address: {:#010x}, Present: {}, Write/Read: {}", self.page_table_address(), self.is_present(), self.is_wr())
    }
}

use crate::page_frame::BITMAP;

pub struct PageDirectory(pub Unique<[PageDirectoryEntry; 1024]>);

impl PageDirectory {
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

    pub fn map_range_pages(&mut self, physical_page_address_start: usize,  physical_address_end: usize, virtual_page_address: usize) {
        let mut i = 0;
        while physical_page_address_start + (i << 12) < physical_address_end  {
            self.map_pages(physical_page_address_start + (i << 12), virtual_page_address + (i << 12));
            i += 1;
        }
    }

    fn map_n_pages(&mut self, physical_page_address: usize, virtual_page_address: usize, n_pages: usize) {
        for i in 0..n_pages {
            self.map_pages(physical_page_address + i*0x1000, virtual_page_address + i*0x1000);
        }
    }

    pub fn map_pages(&mut self, physical_page_address: usize, virtual_page_address: usize) {
        println!("in {:#010x} {:#010x}", physical_page_address, virtual_page_address);
        assert_eq!(0, physical_page_address & 0xFFF, "physical address is not aligned: {:#10x}", physical_page_address);
        assert_eq!(0, virtual_page_address & 0xFFF, "physical address is not aligned: {:#10x}", virtual_page_address);
        assert!(BITMAP.lock().is_available(physical_page_address),
            "frame already used at address {:#10x}", physical_page_address);

        let d_offset = virtual_page_address >> 22;
        let t_offset = (virtual_page_address & 0x3FF000) >> 12;

        let mut page_table: PageTable;
        let page_table_add: usize;
        
        if !self.ref_dir()[d_offset].is_present() {
            BITMAP.lock().get_page_frame(physical_page_address);
            page_table_add = BITMAP.lock().get_available_page_frame();
            page_table = unsafe { PageTable(Unique::new_unchecked(page_table_add as *mut _)) };
            page_table.clear();
            self.mut_dir()[d_offset] = PageDirectoryEntry::new(page_table_add, 0x1);
        } else {
            page_table_add = self.ref_dir()[d_offset].page_table_address();
            page_table = unsafe { PageTable(Unique::new_unchecked(page_table_add as *mut _)) };
            assert!(!page_table.ref_table()[t_offset].is_present(),
                "page table entry was already present with virtual address {:#10x}", virtual_page_address);
            BITMAP.lock().get_page_frame(physical_page_address)
        }
        page_table.mut_table()[t_offset] = PageTableEntry::new(physical_page_address, 0x1);
    }

    pub fn set_entry(&mut self, index: usize, address: usize, tags: usize) {
        self.mut_dir()[index] = PageDirectoryEntry::new(address, tags);
    }

}

impl fmt::Display for PageDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for entry in self.ref_dir() {
            if entry.is_present() {
                if let Err(e) = write!(f, "{}\n", entry) {
                    return Err(e)
                }
            }
        }
        Ok(())
    }
}
