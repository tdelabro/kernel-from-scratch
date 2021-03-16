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
    unsafe {
        asm!("mov eax, ebp", out("eax") base_pointer);
    }

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

    unsafe {
        asm!("lea {}, [stack_high]
            mov {}, esp", 
            out(reg) stack_high, out(reg) esp,
            options(nostack));
    }

    let mut c: usize = 0;
    let mut head = esp;
    while head != stack_high && (c < max || max == 0) {
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
    println!("{}", GdtR::current());
}

pub fn dump_gdt() {
    let mut i = 0;
    while let Some(desc) = GdtR::get_desc(i) {
        println!("Descriptor at index [{}]:\n{}\n", i, desc);
        i += 1;
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
        asm!("mov ax, cs", out("ax") cs, options(nostack));
        asm!("mov ax, ds", out("ax") ds, options(nostack));
        asm!("mov ax, ss", out("ax") ss, options(nostack));
        asm!("mov ax, es", out("ax") es, options(nostack));
        asm!("mov ax, fs", out("ax") fs, options(nostack));
        asm!("mov ax, gs", out("ax") gs, options(nostack));
    }
    println!(
        "cs: {:#06x}, ds: {:#06x}, ss: {:#06x}\nes: {:#06x}, fs: {:#06x}, gs: {:#06x}",
        cs, ds, ss, es, fs, gs
    );
}

use crate::external_symbols::*;

/// Print the addresses of symbols defined in the linker script
///
/// Begining and enf oth the kernel sections and stack
pub fn print_kernel_sections_addresses() {
    println!(
        "kernel: start {:p} end {:p}",
        get_kernel_start(),
        get_kernel_end()
    );
    println!(
        "text: start {:p} end {:p}",
        get_section_text_start(),
        get_section_text_end()
    );
    println!(
        "rodata: start {:p} end {:p}",
        get_section_rodata_start(),
        get_section_rodata_end()
    );
    println!(
        "data: start {:p} end {:p}",
        get_section_data_start(),
        get_section_data_end()
    );
    println!(
        "bss: start {:p} end {:p}",
        get_section_bss_start(),
        get_section_bss_end()
    );
    println!("common bss sep: {:p}", get_common_bss_sep());
    println!(
        "stack: low {:p} high {:p}",
        get_stack_low(),
        get_stack_high()
    );
}

pub fn dump_bitmap() {
    println!("{}", super::physical_memory_management::BITMAP.lock());
}
