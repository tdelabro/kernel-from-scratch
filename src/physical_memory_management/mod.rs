//! Physical page frames management
//!
//! Keep track of the availibility of each physical page frame.
//! Optimize time complexity of finding an available one.

/// 0x1000
pub const PAGE_SIZE_4K: usize = 4096;
/// 0x8000000
pub const RAM_SIZE: usize = 0x8000000;
const N_FRAMES: usize = RAM_SIZE / PAGE_SIZE_4K;
const BITMAP_LEN: usize = N_FRAMES / 32;

/// Bitmap representation of physical memrory
///
/// One bit for each page frame:
/// - Set => in use
/// - Clear => available
pub struct FrameManager {
    bitmap: [u32; BITMAP_LEN],
    skip: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum PhysicalMemoryError {
    NoFrameAvailable,
    FrameAlreadyInUse,
    FrameNotInUse,
}

#[derive(Copy, Clone)]
struct PageFrame(usize);

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
    fn next_available(&self) -> Result<PageFrame, PhysicalMemoryError> {
        let idx = self.bitmap.iter()
            .skip(self.skip)
            .position(|&x| x != !0);

        idx.map_or(Err(PhysicalMemoryError::NoFrameAvailable), |i| {
            let mut j: usize = 0;
            while !self.bitmap[i] & (0x80000000 >> j) == 0 {
                j += 1;
            }
            Ok(PageFrame((i * 32 + j) * PAGE_SIZE_4K))
        })
    }

    fn mark_as_used(&mut self, page: PageFrame) -> Result<(), PhysicalMemoryError> {
        let i = page.index();
        let o = page.offset();

        match self.bitmap[i] & (0x80000000 >> o) == 0 {
            false => Err(PhysicalMemoryError::FrameAlreadyInUse),
            true => {
                self.bitmap[i] |= 0x80000000 >> o;
                self.skip = i;
                Ok(())
            }
        }
    }

    fn mark_as_available(&mut self, page: PageFrame) -> Result<(), PhysicalMemoryError> {
        let i = page.index();
        let o = page.offset();

        match self.bitmap[i] & (0x80000000 >> o) != 0 {
            false => Err(PhysicalMemoryError::FrameNotInUse),
            true => {
                self.bitmap[i] &= !(0x80000000 >> o);
                self.skip = i;
                Ok(())
            }
        }
    }

    pub fn alloc_frame(&mut self) -> Result<usize, PhysicalMemoryError> {
        let p = self.next_available()?;
        self.mark_as_used(p)?;
        Ok(p.address())
    }

    pub fn alloc_frame_by_address(&mut self, address: usize) -> Result<(), PhysicalMemoryError> {
        self.mark_as_used(PageFrame::new(address))
    }

    pub fn free_frame(&mut self, address: usize) -> Result<(), PhysicalMemoryError> {
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
        write!(f, "Used frames:\n")?;
        for (i, u) in self.bitmap.iter().enumerate() {
            for j in 0..32 {
                if u & (0x80000000 >> j) != 0 {
                    write!(f, "{:#010x} ",  PAGE_SIZE_4K * (32 * i + j))?;
                }
            }
        }
        Ok(())
    }
}

use spin::Mutex;

/// Unique source of true for physical memory management
pub static BITMAP: Mutex<FrameManager> = Mutex::new(FrameManager {
    bitmap: [0; BITMAP_LEN],
    skip: 0,
});
