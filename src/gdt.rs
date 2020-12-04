/// Segment Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtDesc {
    lim0_15: u16,
    base0_15: u16,
    base16_23: u8,
    access: u8,
    lim16_19_flags: u8,
    base24_31: u8,
}

impl GdtDesc {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> GdtDesc {
        GdtDesc {
            lim0_15: (limit & 0xffff) as u16,
            base0_15: (base & 0xffff) as u16,
            base16_23: ((base & 0xff0000) >> 16) as u8,
            access: access,
            lim16_19_flags: ((limit & 0xf0000) >> 16) as u8 | (flags & 0xf) << 4,
            base24_31: ((base & 0xff000000) >> 24) as u8,
        }
    }
}

use core::fmt;

impl fmt::Display for GdtDesc {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
            let base: u32 = self.base0_15 as u32
                | (self.base16_23 as u32) << 16 | (self.base24_31 as u32) << 24;
            let limit: u32 = self.lim0_15 as u32
                | ((self.lim16_19_flags & 0xf) as u32) << 16;
            let present: bool = (self.access & 0x80) != 0;
            let privilege: u8 = (self.access & 0x60) >> 5;
            let desc_type: bool = (self.access & 0x10) != 0;
            let executable: bool = (self.access & 0x8) != 0;
            let dc: bool = (self.access & 0x4) != 0;
            let rw: bool = (self.access & 0x2) != 0;
            let accessed: bool = (self.access & 0x1) != 0;
            let granularity: bool = (self.lim16_19_flags & 0x80) != 0;
            let size: bool = (self.lim16_19_flags & 0x40) != 0;

            write!(f, "base: {}, limit: {}\npresent: {}, privilege: {}\ntype {}, dc: {}, rw: {}\naccessed: {}, granularity: {}, size: {}",
                base, limit, present, privilege,
                match desc_type {
                    false => "System",
                    true => match executable {
                        false => "Data",
                        true => "Code",
                    }
                },
                dc, rw, accessed,
                match granularity {
                    false => "1B",
                    true => "4KiB",
                },
                match size {
                    false => "16bit",   
                    true => "32bit",   
                })
        }
}

/// GDT Register
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtR {
    limit: u16,
    base: u32,
}

extern "C" {
    fn memcpy(dst: *mut u8, src: *const u8, size: usize);
}

const GDTBASE: u32 = 0x00000800;
const GDTLEN: usize = 7;

const GDT: [GdtDesc; GDTLEN] = [
    GdtDesc::new(0x0, 0x0, 0x0, 0x0),
    GdtDesc::new(0x0, 0xFFFFF, 0x9A, 0x0D), // Code
    GdtDesc::new(0x0, 0xFFFFF, 0x92, 0x0D), // Data
    GdtDesc::new(0x0, 0x0, 0x96, 0x0D),     // Stack
    GdtDesc::new(0x0, 0xFFFFF, 0xFA, 0x0D), // User Code
    GdtDesc::new(0x0, 0xFFFFF, 0xF2, 0x0D), // User Data
    GdtDesc::new(0x0, 0x0, 0xF6, 0x0D),     // User Stack
];

const GDTR: GdtR = GdtR {
    limit: (8 * GDTLEN - 1) as u16,
    base: GDTBASE,
};

pub fn init() {
    unsafe { 
        memcpy(GDTBASE as *mut u8, GDT.as_ptr() as *const u8, 8 * GDTLEN);
        asm!("lgdtl ({})", in(reg) &GDTR, options(att_syntax));
        asm!(
            "ljmp $0x08, $next",
            "next:",
            "movw $0x10, %ax",
            "movw %ax, %ds",
            "movw %ax, %es",
            "movw %ax, %fs",
            "movw %ax, %gs",
            "movw $0x18, %ax",
            "movw %ax, %ss",
            options(att_syntax),
        );
    }
}
