mod memory_map;
mod framebuffer;

use core::mem;
use core::convert::TryInto;
use self::memory_map::*;
use self::framebuffer::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MultibootInfo {
    pub inner: *const FixedPart,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FixedPart {
    total_size: u32,
    _reserved: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct MultibootInfoIntoIter {
    inner: *const Tag,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Tag {
    typ: usize,
    size: usize,
}

impl Iterator for MultibootInfoIntoIter {
    type Item = *const Tag;
    fn next(&mut self) -> Option<Self::Item> {
        let tag = unsafe { &*(self.inner) };
        if tag.typ == 0 && tag.size == 8 {
            None
        } else {
            let offset = match tag.size {
                s if s % 8 == 0 => s,
                s => (s & !0x7) + 8,
            };
            let ret = self.inner;
            self.inner = (self.inner as usize + offset) as *const Tag;
            Some(ret)
        }
    }
}

impl IntoIterator for MultibootInfo {
    type Item = *const Tag;
    type IntoIter = MultibootInfoIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        MultibootInfoIntoIter {
            inner: unsafe { self.inner.offset(1) as *const Tag },
        }
    }
}

impl MultibootInfo {
    pub fn get_memory_map(&self) -> Option<MemoryMap> {
        for tag in self.into_iter() {
            if unsafe { (*tag).typ } == 6 {
                return Some(MemoryMap {
                    inner: tag as *const MemoryMapTag,
                })
            }
        }

        None
    }

    pub fn get_framebuffer(&self) -> Option<FramebufferTag> {
        for tag in self.into_iter() {
            if unsafe { (*tag).typ } == 8 {
                return Some(unsafe { *(tag as *const FramebufferTag) })
            }
        }

        None
    }
}
