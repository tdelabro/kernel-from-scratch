//! Global Descriptor Table

mod tss;
mod segment_descriptor;

use core::fmt;
use self::tss::Tss;
pub use self::segment_descriptor::SegmentDescriptor;

extern "C" {
    fn memcpy(dst: *mut u8, src: *const u8, size: usize);
}

/// GDT Register
///
/// Memory layout of the 48 bit GDT register
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct GdtR {
    pub limit: u16,
    pub base: usize,
}

impl fmt::Display for GdtR {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
            let base = self.base;
            let limit = self.limit;
            write!(f, "base: {:#010x}, limit: {:#06x}", base, limit)
        }
}

pub const GDTBASE: usize = 0x00000800;
const GDTLEN: usize = 8;

use core::mem::size_of;

impl GdtR {
    pub fn current() -> GdtR {
        let gdtr = GdtR::default();

        unsafe {
            asm!("sgdt [{}]", in(reg) &gdtr as *const _);
        }
        gdtr
    }

    pub fn get_desc(index: usize) -> Option<SegmentDescriptor> {
        let gdtr = GdtR::current();
        let gdt_len = (gdtr.limit as usize + 1) / 8;
        if index >= gdt_len {
            return None
        }
        let mut desc: SegmentDescriptor = Default::default();
        unsafe {
            memcpy(&mut desc as *mut _ as *mut u8, (gdtr.base + size_of::<SegmentDescriptor>() * index) as *const u8, 8);
        }
        Some(desc)
    }
}

/// Initialize the Global Descriptor Table
///
/// The GDT is setup with those hardcoded segments:  
/// GDT\[0\] = Null  
/// GDT\[1\] = Kernel Code  
/// GDT\[2\] = Kernel Data  
/// GDT\[3\] = Kernel Stack  
/// GDT\[4\] = User Code  
/// GDT\[5\] = User Data  
/// GDT\[6\] = Usert Stack  
/// GDT\[7\] = Task State Segment  
pub fn init() {
    let stack_high: u32;
    unsafe { asm!("lea {}, [stack_high]", out(reg) stack_high, options(nostack)); }
    let tss = Tss::new(stack_high); 

    let descriptors: [SegmentDescriptor; GDTLEN] = [
        SegmentDescriptor::new(0x0, 0x0, 0x0, 0x0),       // 0x0 Not used

        SegmentDescriptor::new(0x0, 0xFFFFF, 0x9A, 0x0D), // 0x8  Code 
        SegmentDescriptor::new(0x0, 0xFFFFF, 0x92, 0x0D), // 0x10 Data
        SegmentDescriptor::new(0x0, 0x0, 0x96, 0x0D),     // 0x18 Stack

        SegmentDescriptor::new(0x0, 0xFFFFF, 0xFE, 0x0D), // 0x20 User Code
        SegmentDescriptor::new(0x0, 0xFFFFF, 0xF2, 0x0D), // 0x28 User Data
        SegmentDescriptor::new(0x0, 0x0, 0xF6, 0x0D),     // 0x30 User Stack

        SegmentDescriptor::new(&tss as *const Tss as u32, 0x67, 0xE9, 0x00),   // 0x38 TSS
    ];

    let mut gdt = Gdt {
        base: GDTBASE,
        len: 0,
    };
    gdt.init(&descriptors);
}

struct Gdt {
    base: usize,
    len: usize,
}

impl Gdt {
    unsafe fn copy_to_base(&mut self, gdt: &[SegmentDescriptor]) {
        memcpy(self.base as *mut u8, gdt.as_ptr() as *const u8, 8 * gdt.len());
        self.len = gdt.len();
    }

    fn generate_gdtr(&self) -> GdtR {
        GdtR {
            limit: (8 * self.len - 1) as u16,
            base: self.base,
        }
    }

    unsafe fn load_to_reg(&self) {
        let gdtr = self.generate_gdtr();
        asm!("  lgdtl ({})
                ljmp $0x08, $1f
            1:
                movw $0x18, %ax
                movw %ax, %ss
                movw $0x10, %ax
                movw %ax, %ds
                movw %ax, %es
                movw %ax, %fs
                movw %ax, %gs
                movw $0x38, %ax
                ltr %ax",
            in(reg) &gdtr,
            out("ax") _,
            options(att_syntax),
        );
    }

    fn init(&mut self, gdt: &[SegmentDescriptor]) {
        unsafe {
            self.copy_to_base(gdt);
            self.load_to_reg();
        }
    }
}
