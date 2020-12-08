#[inline(always)]
pub fn stack_trace(max_frame: usize) {
    let mut base_pointer: *const usize;
    unsafe { asm!("mov eax, ebp", out("eax") base_pointer); }

    println!("Stack Trace:");
    let mut c: usize = 0;
    while !base_pointer.is_null() && c < max_frame {
        let return_address = unsafe { *(base_pointer.offset(1)) } as usize;
        println!("Called in: {:#010x}", return_address);
        base_pointer = unsafe { (*base_pointer) as *const usize };
        c += 1;
    }
    print!("\n");
}

use crate::gdt::GdtR;

pub fn dump_segment_registers() {
    let mut gdtr = GdtR {
        limit: 0,
        base: 0,
    };
    let cs: u16;
    let ds: u16;
    let ss: u16;
    let es: u16;
    let fs: u16;
    let gs: u16;

    let x = gdtr as *const _;
    unsafe {
        asm!("sgdt ({})", out(reg) x, options(att_syntax));
        asm!("mov ax, cs", out("ax") cs);
        asm!("mov ax, ds", out("ax") ds);
        asm!("mov ax, ss", out("ax") ss);
        asm!("mov ax, es", out("ax") es);
        asm!("mov ax, fs", out("ax") fs);
        asm!("mov ax, gs", out("ax") gs);
    }
    println!("gdtr {:?}", gdtr);

    /*
    unsafe {
        let gdtr: &GdtR = match (r_gdtr as *const GdtR).as_ref() {
            Some(p) => p,
            None => {
                println!("Error: gdtr point to null");
                return;
            }
        };
        println!("GDTR\nlimit: {:#06x}, base: {:#010x}\n", gdtr.limit, gdtr.base);
    }
    */
    println!("Segment Registers\ncs: {:#04x}, ds: {:#04x}, ss: {:#04x}\nes: {:#04x}, fs: {:#04x}, gs: {:#04x}\n",
        cs, ds, ss, es, fs, gs);
}
