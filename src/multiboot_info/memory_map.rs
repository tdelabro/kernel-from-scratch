#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryMapTag {
    typ: usize,
    size: usize,
    entry_size: u32,
    pub entry_version: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub typ: u32,
    _reserved: u32,
}

#[derive(Debug)]
pub struct MemoryMap {
    pub inner: *const MemoryMapTag,
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryMapIntoIter {
    inner: *const MemoryMapEntry,
    end: *const MemoryMapEntry,
}

impl IntoIterator for MemoryMap {
    type Item = *const MemoryMapEntry;
    type IntoIter = MemoryMapIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            MemoryMapIntoIter {
                inner: self.inner.offset(1) as *const MemoryMapEntry,
                end: (self.inner as usize + (*self.inner).size as usize) as *const MemoryMapEntry,
            }
        }
    }
}

impl Iterator for MemoryMapIntoIter {
    type Item = *const MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner >= self.end {
            None
        } else {
            let ret = self.inner;
            self.inner = unsafe { self.inner.offset(1) };
            Some(ret)
        }
    }
}
