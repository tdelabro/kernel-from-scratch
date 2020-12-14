const PAGE_DIR_ADDRESS: usize = 0x21000;
const FIRST_PAGE_TABLE: usize = 0x400000;

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

pub struct PageDirectory(Unique<[usize; 1024]>);

impl core::convert::AsRef<[usize; 1024]> for PageDirectory {
   fn as_ref(&self) -> &[usize; 1024] {
        unsafe { self.0.as_ref() }
   }
}

impl core::convert::AsMut<[usize; 1024]> for PageDirectory {
   fn as_mut(&mut self) -> &mut [usize; 1024] {
        unsafe { self.0.as_mut() }
   }
}

impl PageDirectory {
    pub fn init_identity(&mut self) {
        self.as_mut().iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = (FIRST_PAGE_TABLE + i*4*1024) | 3);

        let y: &mut [[usize; 1024]; 1024]  = unsafe {
            core::mem::transmute::<usize, &mut [[usize; 1024]; 1024]>(FIRST_PAGE_TABLE)
        };
        for i in 0..1024 {
            for j in 0..1024 {
                y[i][j] = ((i*1024 + j)*4096) | 3;
            }
        }
        translate(0x0);
        translate(0x10000);
        translate(0x123456b);
    }
}

fn translate(add: u32) {
    let d_offset = add >> 22;
    let t_offset = (add & 0x3FF000) >> 12;
    let p_offset = add & 0xFFF;
    println!("{} {} {}", d_offset, t_offset, p_offset);
    unsafe {
        let d_entry: u32 = *((PAGE_DIR_ADDRESS as u32 + d_offset*4) as *const u32);
        let t_entry: u32 = *(((d_entry & 0xFFFFF000) + t_offset*4) as *const u32);
        let padd: u32 = (t_entry & 0xFFFFF000) + p_offset;
        println!("d_entry: {:#x} t_entry: {:#x}", d_entry, t_entry);
        println!("vadd {:#x} padd {:#x}", add, padd);
    }
}

pub static PAGE_DIRECTORY: Mutex<PageDirectory> = Mutex::new(PageDirectory(
        unsafe { Unique::new_unchecked(PAGE_DIR_ADDRESS as *mut _) }));

