#[repr(C, packed)]
pub struct Tss {
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
    pub fn new(esp0: u32) -> Tss {
        Tss {
            link: 0, link_h: 0,
            esp0: esp0,
            ss0: 0x18, ss0_h: 0,
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
            es: 0x13, es_h: 0, // 0x10 | 0x3
            cs: 0x0b, cs_h: 0, // 0x8 | 0x3
            ss: 0x13, ss_h: 0,
            ds: 0x13, ds_h: 0,
            fs: 0x13, fs_h: 0,
            gs: 0x13, gs_h: 0,
            ldtr: 0, ldtr_h: 0,
            trap: 0, oipb_offset: 0x68, // 0x68 = 104 = size_of(Tss)
        }
    }
}
