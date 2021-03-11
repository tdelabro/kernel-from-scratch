#[repr(C)]
#[derive(Debug)]
#[derive(Copy, Clone)]
struct BasicMemoryTag {
    typ: usize,
    size: usize,
    mem_lower: u32,
    mem_upper: u32,
}

unsafe fn parse_basic_memory(tag: *const BasicMemoryTag) {
    println!("{:?}", *tag)
}
