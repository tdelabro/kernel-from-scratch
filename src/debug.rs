#[inline(always)]
pub fn stack_trace(max: usize) {
    let mut base_pointer: *const usize;
    unsafe { asm!("mov eax, ebp", out("eax") base_pointer); }

    let mut c: usize = 0;
    while !base_pointer.is_null() && (c < max || max == 0) {
        let return_address = unsafe { *(base_pointer.offset(1)) } as usize;
        println!("Called in: {:#010x}", return_address);
        base_pointer = unsafe { (*base_pointer) as *const usize };
        c += 1;
    }
}

pub fn dump_stack(max: usize) {
    let stack_high: *const usize;
    let esp: *const usize;

    unsafe { asm!("lea {}, [stack_high]", out(reg) stack_high); }
    unsafe { asm!("mov {}, esp", out(reg) esp); }

    let mut c: usize = 0;
    let mut head = esp;
    while head != stack_high && (c < max || max == 0){
        println!("{:p}: {:#010x}", head, unsafe { *head } as usize);
        head = unsafe { head.offset(1) };
        c += 1;
    }
    println!("Stack size: {}", stack_high as usize - esp as usize);
}

use crate::gdt::GdtR;

pub fn dump_gdtr() {
    let mut gdtr = GdtR {
        limit: 0,
        base: 0,
    };

    unsafe {
        asm!("sgdt [{}]", in(reg) &mut gdtr as *const _);
        println!("limit: {:#06x}, base: {:#010x}", gdtr.limit, gdtr.base);
    }
}

pub fn dump_segment_registers() {
    let cs: u16;
    let ds: u16;
    let ss: u16;
    let es: u16;
    let fs: u16;
    let gs: u16;

    unsafe {
        asm!("mov ax, cs", out("ax") cs);
        asm!("mov ax, ds", out("ax") ds);
        asm!("mov ax, ss", out("ax") ss);
        asm!("mov ax, es", out("ax") es);
        asm!("mov ax, fs", out("ax") fs);
        asm!("mov ax, gs", out("ax") gs);
    }
    println!("cs: {:#04x}, ds: {:#04x}, ss: {:#04x}\nes: {:#04x}, fs: {:#04x}, gs: {:#04x}",
        cs, ds, ss, es, fs, gs);
}
