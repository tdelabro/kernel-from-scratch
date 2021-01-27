const N_PAGES: usize = 1024 * 1024;
const BITMAP_LEN: usize = N_PAGES / 32;
const PAGE_SIZE_4K: usize = 4096;

pub struct FrameManager {
    bitmap: [u32; BITMAP_LEN],
    skip: usize,
}

#[derive(Copy, Clone)]
pub struct PageFrame(usize);

impl PageFrame {
    fn new(address: usize) -> PageFrame {
        assert_eq!(0, address & 0xFFF, "frame address is not aligned: {:#10x}", address);
        PageFrame(address)
    }

    fn index(&self) -> usize {
        self.0 / PAGE_SIZE_4K / 32
    }

    fn offset(&self) -> usize {
        self.0 / PAGE_SIZE_4K % 32
    }

    fn address(&self) -> usize {
        self.0
    }
}

impl FrameManager {
    fn next_available(&self) -> PageFrame {
        let i = self.bitmap.iter()
            .skip(self.skip)
            .position(|&x| x != !0)
            .unwrap();
        let mut j: usize = 0;

        while !self.bitmap[i] & (0x80000000 >> j) == 0 {
            j += 1;
        }
        PageFrame((i * 32 + j) * PAGE_SIZE_4K)
    }

    fn mark_as_used(&mut self, page: PageFrame) {
        let i = page.index();
        let o = page.offset();

        assert!(self.bitmap[i] & (0x80000000 >> o) == 0);

        self.bitmap[i] |= 0x80000000 >> o;
        self.skip = i;
    }
    fn mark_as_available(&mut self, page: PageFrame) {
        let i = page.index();
        let o = page.offset();

        assert!(self.bitmap[i] & (0x80000000 >> o) != 0);

        self.bitmap[i] &= !(0x80000000 >> o);
        self.skip = i;
    }

    pub fn kalloc_frame(&mut self) -> usize {
        let p = self.next_available();
        self.mark_as_used(p);
        p.address()
    }

    pub fn kalloc_frame_by_address(&mut self, address: usize) {
        self.mark_as_used(PageFrame::new(address));
    }

    pub fn free_frame(&mut self, address: usize) {
        self.mark_as_available(PageFrame::new(address))
    }

    pub fn is_available(&self, address: usize) -> bool {
        let page = PageFrame::new(address);

        self.bitmap[page.index()] & (0x80000000 >> page.offset()) == 0
    }
}

use core::fmt;

impl fmt::Display for FrameManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Err(e) =  write!(f, "Used frames:\n") {
            return Err(e);
        }
        for (i, u) in self.bitmap.iter().enumerate() {
            for j in 0..32 {
                if u & (0x80000000 >> j) != 0 {
                    if let Err(e) =  write!(f, "{:#010x} ",  PAGE_SIZE_4K * (32 * i + j)) {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}

use spin::Mutex;

pub static BITMAP: Mutex<FrameManager> = Mutex::new(FrameManager {
    bitmap: [0; BITMAP_LEN],
    skip: 0,
});
