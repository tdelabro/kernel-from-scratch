use core::mem;
use core::convert::TryInto;

#[derive(Debug, Default)]
struct MultibootInfo {
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Tag {
    typ: usize,
    size: usize,
}

fn get_next_tag(current: *const Tag) -> Option<*const Tag> {
    let tag = unsafe { current.read() };
    if tag.typ == 0 && tag.size == 8 {
        None
    } else {
        let offset = match tag.size {
            s if s % 8 == 0 => s,
            s => (s & !0x7) + 8,
        };
        Some((current as usize + offset) as *const Tag)
    }
}

pub unsafe fn parse_multiboot_info (magic_number: usize, p_info: *const usize) {
    let mut ret: MultibootInfo = Default::default();
    let fixed_size_part = p_info as *const Tag;
    assert_eq!(magic_number, 0x36d76289, "System hadn't been loaded by a Multiboot2-compliant boot loader.");
    println!("Parsing Multiboot info");
    println!("Total size: {}, start: {:p}", (*fixed_size_part).typ, fixed_size_part);
    
    if (*fixed_size_part).typ <= 8 {
        return
    }

    let mut head = Some(fixed_size_part.offset(8));

    let mut c = 0;
    while let Some(current) = head {
        println!("{} {:p} {:?}", c, current, *current);
        
        match (*current).typ {
            8 => parse_framebuffer(current as *const FramebufferTag, &mut ret),
            _ => {},
        }

        c += 1;
        head = get_next_tag(current);
    }

    println!("{:?}", ret)
}

#[derive(Debug)]
#[derive(Copy, Clone)]
struct FramebufferTag {
    typ: usize,
    size: usize,
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    reserved: u8,
}

fn parse_framebuffer(tag: *const FramebufferTag, ret: &mut MultibootInfo) {
    unsafe {
        ret.framebuffer_addr = (*tag).framebuffer_addr; 
        ret.framebuffer_pitch = (*tag).framebuffer_pitch; 
        ret.framebuffer_width = (*tag).framebuffer_width; 
        ret.framebuffer_height = (*tag).framebuffer_height; 
        ret.framebuffer_bpp = (*tag).framebuffer_bpp; 
        ret.framebuffer_type = (*tag).framebuffer_type; 
    }
}
