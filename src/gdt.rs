const GDTBASE: u32 = 0x00000800;
const GDTLEN: usize = 7;

/// Segment Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtDesc {
    lim0_15: u16,
    base0_15: u16,
    base16_23: u8,
    acces: u8,
    lim16_19_flags: u8,
    base24_31: u8,
}

impl GdtDesc {
    pub const fn new(base: u32, limit: u32, acces: u8, flags: u8) -> GdtDesc {
        GdtDesc {
            lim0_15: (limit & 0xffff) as u16,
            base0_15: (base & 0xffff) as u16,
            base16_23: ((base & 0xff0000) >> 16) as u8,
            acces: acces,
            lim16_19_flags: ((limit & 0xf0000) >> 16) as u8 | (flags & 0xf) << 4,
            base24_31: ((base & 0xff000000) >> 24) as u8,
        }
    }
}

/// GDT Register
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtR {
    limit: u16,
    base: u32,
}

extern "C" {
    fn memcpy(dst: *mut u8, src: *const u8, size: usize);
}

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
            "movw $0x10, %ax",
            "movw %ax, %ds",
            "movw %ax, %es",
            "movw %ax, %fs",
            "movw %ax, %gs",
            "ljmp $0x08, $next",
            "next:",
            options(att_syntax),
        );
    }
}
