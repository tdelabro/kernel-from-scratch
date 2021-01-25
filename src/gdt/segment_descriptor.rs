/// Segment Descriptor
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct SegmentDescriptor {
    lim0_15: u16,
    base0_15: u16,
    base16_23: u8,
    access: u8,
    lim16_19_flags: u8,
    base24_31: u8,
}

impl SegmentDescriptor {
    pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> SegmentDescriptor {
        SegmentDescriptor {
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

impl fmt::Display for SegmentDescriptor {
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

            write!(f, "base: {:#010x}, limit: {}\npresent: {}, privilege: {}\ntype {}, dc: {}, rw: {}\naccessed: {}, granularity: {}, size: {}",
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
