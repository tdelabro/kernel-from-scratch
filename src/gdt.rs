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

#[repr(C, packed)]
struct Tss {
    link: u16, link_h: u16,
    esp0: u32,
    ss0: u16, ss0_h: u16,
    esp1: u32,
    ss1: u16, ss1_h: u16,
    esp2: u32,
    ss2: u16, ss2_h: u16,
    cr3: u32,
    eip: u32,
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    es: u16, es_h: u16,
    cs: u16, cs_h: u16,
    ss: u16, ss_h: u16,
    ds: u16, ds_h: u16,
    fs: u16, fs_h: u16,
    gs: u16, gs_h: u16,
    ldtr: u16, ldtr_h: u16,
    trap: u16, oipb_offset: u16,
}

impl Tss {
    fn new(ss: u16, esp: u32, iopb: u16) -> Tss {
        Tss {
            link: 0, link_h: 0,
            esp0: esp,
            ss0: ss, ss0_h: 0,
            esp1: 0,
            ss1: 0, ss1_h: 0,
            esp2: 0,
            ss2: 0, ss2_h: 0,
            cr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0x13, es_h: 0,
            cs: 0x0b, cs_h: 0,
            ss: 0x13, ss_h: 0,
            ds: 0x13, ds_h: 0,
            fs: 0x13, fs_h: 0,
            gs: 0x13, gs_h: 0,
            ldtr: 0, ldtr_h: 0,
            trap: 0, oipb_offset: iopb,
        }
    }
}


const GDTBASE: u32 = 0x00000800;
const GDTLEN: usize = 8;

const GDTR: GdtR = GdtR {
    limit: (8 * GDTLEN - 1) as u16,
    base: GDTBASE,
};

pub fn init() {
    let stack_high: u32;
    unsafe { asm!("mov {}, stack_high", out(reg) stack_high); }

    let tss = Tss::new(0x18, stack_high, 0x68); // 0x68 = 104 = size_of(Tss)
    let gdt: [GdtDesc; GDTLEN] = [
        GdtDesc::new(0x0, 0x0, 0x0, 0x0),
        GdtDesc::new(0x0, 0xFFFFF, 0x9A, 0x0D), // 0x8  Code 
        GdtDesc::new(0x0, 0xFFFFF, 0x92, 0x0D), // 0x10 Data
        GdtDesc::new(0x0, 0x0, 0x96, 0x0D),     // 0x18 Stack

        GdtDesc::new(0x0, 0xFFFFF, 0xFA, 0x0D), // 0x20 User Code
        GdtDesc::new(0x0, 0xFFFFF, 0xF2, 0x0D), // 0x28 User Data
        GdtDesc::new(0x0, 0x0, 0xF6, 0x0D),     // 0x30 User Stack

        GdtDesc::new(&tss as *const Tss as u32, 0x67, 0xE9, 0x00),   // 0x38 TSS
    ];

    unsafe { 
        memcpy(GDTBASE as *mut u8, gdt.as_ptr() as *const u8, 8 * GDTLEN);
        asm!("lgdtl ({})", in(reg) &GDTR, options(att_syntax));
        asm!("
            ljmp $0x08, $2f
            2:
            movw $0x18, %ax
            movw %ax, %ss
            movw $0x10, %ax
            movw %ax, %ds
            movw %ax, %es
            movw %ax, %fs
            movw %ax, %gs
            movw $0x38, %ax
            ltr %ax
            ", options(att_syntax),
        );
    }
}
