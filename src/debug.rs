//! Debug tools
//!
//! Check the system internal memory state at runtime.

/// Display the stack trace
///
/// Each line contains the return address of a nested function call.
/// They are displayed from the most to less deep.
#[inline(always)]
pub fn stack_trace(max: usize) {
    let mut base_pointer: *const usize;
    unsafe { asm!("mov eax, ebp", out("eax") base_pointer); }

    let mut c: usize = 0;
    while !base_pointer.is_null() && (c < max || max == 0) {
        let return_address = unsafe { *(base_pointer.offset(1)) } as usize;
        println!("{}: {:#010x}", c, return_address);
        base_pointer = unsafe { (*base_pointer) as *const usize };
        c += 1;
    }
}

/// Display the stack
///
/// Print the stack, from top to bottom, up to `max` addresses.
/// Each line follow this format:  
/// \<address on the stack\>: \<content of this address\>  
/// At the end the heigth of the stack is also printed.
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

/// Display the Global Descriptor Table Register
///
/// Print it's base and limit values.
pub fn dump_gdtr() {
    let gdtr = GdtR::default();

    unsafe {
        asm!("sgdt [{}]", in(reg) &gdtr as *const _);
        println!("base: {:#010x}, limit: {:#06x}", gdtr.base, gdtr.limit);
    }
}

/// Display the Segment Registers
///
/// CS, DS, SS, ES, FS and GS values are printed.
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
    println!("cs: {:#06x}, ds: {:#06x}, ss: {:#06x}\nes: {:#06x}, fs: {:#06x}, gs: {:#06x}",
        cs, ds, ss, es, fs, gs);
}
