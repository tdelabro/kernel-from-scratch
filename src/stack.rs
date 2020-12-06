pub fn tmp(id: usize) {
    let ebp: u32;
    let esp: u32;
    let eip: u32;

    unsafe {
        asm!(
            "mov {}, ebp",
            "mov {}, esp",
            "mov {}, [esp]",
            out(reg) ebp,
            out(reg) esp,
            out(reg) eip,
        );
        println!("#{}: ebp {}   esp {}  eip {}", id, ebp, esp, eip);
    }
}

#[derive(Debug)]
#[repr(C)]
struct StackFrame {
    ebp: *const StackFrame,
    eip: u32,
}

pub fn print_stack() {
    let mut base_pointer: *const usize;
    unsafe {
        asm!(
            "mov eax, ebp",
            out("eax") base_pointer,
        );
    }
    println!("ebp {:p}", base_pointer);
    while !base_pointer.is_null() {
        let return_address = unsafe { *(base_pointer.offset(1)) } as usize;
        println!("Call site: {}", return_address);
        base_pointer = unsafe { (*base_pointer) as *const usize };
    }

    /*
       let ebp: u32;
       let mut stk: *const StackFrame;

    //stk = core::mem::transmute::<u32, &StackFrame<'_>>(ebp);
    println!("ebp {}", ebp);
    stk = ebp as *const StackFrame;
    unsafe {
    while let Some(head) = stk.as_ref() {
    println!("stackframe: {:p} {:#x}", head.ebp, head.eip);
    stk = head.ebp as *const StackFrame;
    }
    }
    println!("stk {:p}", stk);
    */
    /*
       println!("stackframe: {:p} {:#x}", (*stk).ebp, (*stk).eip);
       while let Some() {
       println!("stackframe: {:p} {:#x}", stk.ebp, stk.eip);
       stk = stk.ebp;
       }
       */
}
