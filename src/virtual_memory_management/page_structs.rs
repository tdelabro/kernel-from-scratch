use core::ptr::Unique;
use core::fmt;

use crate::physical_memory_management::PhysicalMemoryError;

#[derive(Debug)]
pub enum VirtualMemoryError {
    PhysicalMemoryError(PhysicalMemoryError),
}

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

    fn set_entry(&mut self, index: usize, address: usize, flags: usize) {
        self.mut_table()[index] = PageTableEntry::new(address, flags);
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
    pub fn is_enabled(&self) -> bool {
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

    fn get_table_linear_add(&self, offset: usize) -> usize {
        match self.is_enabled() {
            true => 0xFFC00000usize + (offset << 12),
            false => self.ref_dir()[offset].page_table_address(),
        }
    }

    pub fn set_entry(&mut self, index: usize, address: usize, flags: usize) {
        self.mut_dir()[index] = PageDirectoryEntry::new(address, flags);
    }

    pub fn map_pages(&mut self, physical_page_address: usize, virtual_page_address: usize, flags: usize)
        -> Result<(), VirtualMemoryError> {
        assert_eq!(0, physical_page_address & 0xFFF, "physical address is not 4k aligned: {:#10x}", physical_page_address);
        assert_eq!(0, virtual_page_address & 0xFFF, "physical address is not 4k aligned: {:#10x}", virtual_page_address);

        let d_offset = virtual_page_address >> 22;
        let t_offset = (virtual_page_address & 0x3FF000) >> 12;

        assert!(d_offset != 1023,
            "trying to map page at: {}. All addresses over 0xFFC00000 are reserved by the self referencing directory trick.",
            virtual_page_address);

        let mut page_table: PageTable;
        let page_table_add: usize;
        if !self.ref_dir()[d_offset].is_present() {
            page_table_add = BITMAP.lock().alloc_frame().map_err(|e| VirtualMemoryError::PhysicalMemoryError(e))?;
            self.set_entry(d_offset, page_table_add, 0x3);
            page_table = unsafe { PageTable(Unique::new_unchecked(self.get_table_linear_add(d_offset) as *mut _)) };
            page_table.clear();
        } else {
            page_table = unsafe { PageTable(Unique::new_unchecked(self.get_table_linear_add(d_offset) as *mut _)) };
        }
        assert!(!page_table.ref_table()[t_offset].is_present(), "page entry already present at index {} for virtual address {}",
            t_offset, virtual_page_address);
        page_table.set_entry(t_offset, physical_page_address, flags);
        Ok(())
    }

    pub fn unmap_pages(&mut self, virtual_page_address: usize) -> Result<(), VirtualMemoryError> {
        assert_eq!(0, virtual_page_address & 0xFFF, "physical address is not 4k aligned: {:#10x}", virtual_page_address);
         
        let d_offset = virtual_page_address >> 22;
        let t_offset = (virtual_page_address & 0x3FF000) >> 12;

        assert!(self.ref_dir()[d_offset].is_present(), "directory entry not present at index {} for virtual address {}",
            d_offset, virtual_page_address);
        let mut page_table = unsafe { PageTable(Unique::new_unchecked(self.get_table_linear_add(d_offset)  as *mut _)) };
        assert!(page_table.ref_table()[t_offset].is_present(), "page entry not present at index {} for virtual address {}",
            t_offset, virtual_page_address);
        BITMAP.lock()
            .free_frame(page_table.ref_table()[t_offset].page_frame_address())
            .map_err(|e| VirtualMemoryError::PhysicalMemoryError(e))?;
        page_table.set_entry(t_offset, 0x0, 0x0);
        Ok(())
    }
}

impl fmt::Display for PageDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, entry) in self.ref_dir().iter().enumerate().filter(|(_, e)| e.is_present()) {
            write!(f, "{:04}: {}\n", idx, entry)?
        }
        Ok(())
    }
}
