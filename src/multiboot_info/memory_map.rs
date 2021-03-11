
#[derive(Debug)]
pub struct MemoryMap {
    pub inner: *const MemoryMapTag
}

#[repr(C)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct MemoryMapTag {
    typ: usize,
    size: usize,
    entry_size: u32,
    entry_version: u32,
}


#[repr(C)]
#[derive(Debug)]
#[derive(Copy, Clone)]
struct MemoryMapEntry {
    base_addr: u64,
    length: u64,
    typ: u32,
    _reserved: u32,
}

unsafe fn parse_memory_map(tag: *const MemoryMapTag) {
    let end = tag as usize + (*tag).size as usize;
    let mut head = tag.offset(1) as *const MemoryMapEntry;

    while (head as usize) < end {
        println!("{:0x?}", *head);
        head = head.offset(1);
    }
    println!("{:?}", *tag);
}
