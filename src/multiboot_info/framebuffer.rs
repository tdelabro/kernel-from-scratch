#[repr(C)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct FramebufferTag {
    typ: usize,
    size: usize,
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    _reserved: u8,
}

unsafe fn parse_framebuffer(tag: *const FramebufferTag) {
    println!("{:?}", tag);
}

