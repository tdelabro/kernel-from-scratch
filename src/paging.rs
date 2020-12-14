const PAGE_DIR_ADDRESS: usize = 0x21000;
const FIRST_PAGE_TABLE_ADDRESS: usize = 0x400000;

pub fn enable() {
    unsafe {
        asm!("mov cr3, eax
            mov ebx, cr0
            or ebx, 0x80000000
            mov cr0, ebx",
            in("eax") PAGE_DIR_ADDRESS, out("ebx") _,
            options(nostack));
    }
}

use spin::Mutex;
use core::ptr::Unique;

struct PageDirectoryEntry(u32);

impl PageDirectoryEntry {
    fn new(address: u32, flags: u32) -> PageDirectoryEntry {
        assert_eq!(0, address & 0xFFF);
        PageDirectoryEntry(address | flags)
    }

    fn page_table_address(&self) -> usize {
        (self.0 & 0xFFFFF000) as usize
    }

    fn present(&self) -> bool {
        self.0 & 0x1 != 0
    }

    fn read_write(&self) -> bool {
        self.0 & (0x1 << 1) != 0
    }
}

pub struct PageManager(Unique<[usize; 1024]>, Unique<[[usize; 1024]; 1024]>);

impl PageManager {
    fn page_directory(&self) -> Unique<[usize; 1024]> {
        self.0
    }

    fn page_tables(&self) -> Unique<[[usize; 1024]; 1024]> {
        self.1
    }
}

impl PageManager {
    pub unsafe fn init_identity(&mut self) {
        for i in 0..1024 {
            self.page_directory().as_mut()[i] = 
                &self.page_tables().as_ref()[i] as *const _ as usize | 3;
            for j in 0..1024 {
                self.page_tables().as_mut()[i][j] = ((i*1024 + j)*4096) | 3;
            }
        }
        assert_eq!(0, self.v_to_p(0));
        assert_eq!(0x1000, self.v_to_p(0x1000));
        assert_eq!(0x8823EF, self.v_to_p(0x8823EF));
    }

    fn v_to_p(&self, v: usize) -> usize {
        let d_offset = v >> 22;
        let t_offset = (v & 0x3FF000) >> 12;
        let p_offset = v & 0xFFF;

        let d_entry = unsafe { self.page_directory().as_ref()[d_offset] };
        let t_entry = unsafe {
            (*((d_entry & 0xFFFFF000) as *const [u32; 1024]))[t_offset]
        };
        (t_entry & 0xFFFFF000) as usize + p_offset 
    }
}

pub static PAGE_DIRECTORY: Mutex<PageManager> = Mutex::new(PageManager(
        unsafe { Unique::new_unchecked(PAGE_DIR_ADDRESS as *mut _) },
        unsafe { Unique::new_unchecked(FIRST_PAGE_TABLE_ADDRESS as *mut _) }));

