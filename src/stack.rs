#[inline(always)]
pub fn trace(max_frame: usize) {
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
    println!("");
}
