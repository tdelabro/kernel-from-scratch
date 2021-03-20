#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FramebufferTag {
    typ: usize,
    size: usize,
    pub framebuffer_addr: u64,
    framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    _reserved: u8,
}
