const N_PAGES: usize = 1024 * 1024;
const BITMAP_LEN: usize = N_PAGES / 32;
const PAGE_SIZE_4K: usize = 4096;

pub struct FrameManager {
    bitmap: [u32; BITMAP_LEN],
    skip: usize,
}

impl FrameManager {
    pub fn malloc(&mut self) -> usize {
        let i = self.bitmap.iter()
            .skip(self.skip)
            .position(|&x| x != !0)
            .unwrap();
        let mut j: usize = 0;

        while !self.bitmap[i] & (0x80000000 >> j) == 0 {
            j += 1;
        }
        self.bitmap[i] |= 0x80000000 >> j;
        self.skip = i;
        return (i * 32 + j) * PAGE_SIZE_4K;
    }

    pub fn free(&mut self, page_id: usize) {
        let i = page_id / PAGE_SIZE_4K / 32;
        let j = page_id / PAGE_SIZE_4K % 32;

        assert!(self.bitmap[i] & (0x80000000 >> j) != 0);

        self.bitmap[i] &= !(0x80000000 >> j);
        self.skip = i;
    }
}

use spin::Mutex;

pub static BITMAP: Mutex<FrameManager> = Mutex::new(FrameManager {
    bitmap: [0; BITMAP_LEN],
    skip: 0,
});
