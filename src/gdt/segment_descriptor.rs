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
            access,
            lim16_19_flags: ((limit & 0xf0000) >> 16) as u8 | (flags & 0xf) << 4,
            base24_31: ((base & 0xff000000) >> 24) as u8,
        }
    }

    fn base(&self) -> usize {
        self.base0_15 as usize | (self.base16_23 as usize) << 16 | (self.base24_31 as usize) << 24
    }

    fn limit(&self) -> usize {
        self.lim0_15 as usize | ((self.lim16_19_flags & 0xf) as usize) << 16
    }

    fn present(&self) -> bool {
        self.access & 0x80 != 0
    }

    fn privilege(&self) -> u8 {
        (self.access & 0x60) >> 5
    }

    fn desc_type(&self) -> bool {
        self.access & 0x10 != 0
    }

    fn executable(&self) -> bool {
        self.access & 0x8 != 0
    }

    fn direction_conforming(&self) -> bool {
        self.access & 0x4 != 0
    }

    fn readable_writable(&self) -> bool {
        self.access & 0x2 != 0
    }

    fn accessed(&self) -> bool {
        self.access & 0x1 != 0
    }

    fn granularity(&self) -> bool {
        self.lim16_19_flags & 0x80 != 0
    }

    fn operand_size(&self) -> bool {
        self.lim16_19_flags & 0x40 != 0
    }

    fn long(&self) -> bool {
        self.lim16_19_flags & 0x20 != 0
    }

    fn available(&self) -> bool {
        self.lim16_19_flags & 0x10 != 0
    }
}

use core::fmt;

impl fmt::Display for SegmentDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "base: {:#010x}, limit: {:#07x}\npresent: {}, privilege: {}\ntype {}, dc: {}, rw: {}\naccessed: {}, granularity: {}, size: {}, long: {}, available {}",
                self.base(), self.limit(), self.present(), self.privilege(),
                match self.desc_type() {
                    false => "System",
                    true => match self.executable() {
                        false => "Data",
                        true => "Code",
                    }
                },
                self.direction_conforming(), self.readable_writable(), self.accessed(),
                match self.granularity() {
                    false => "1B",
                    true => "4KiB",
                },
                match self.operand_size(){
                    false => "16bit",   
                    true => "32bit",   
                },
                self.long(), self.available())
    }
}
