const PAGE_DIR_ADDRESS: usize = 0x21000;
const FIRST_PAGE_TABLE: usize = 0x400000;

fn enable_paging() {
    unsafe {
        asm!("
            mov eax, {}
            mov cr3, eax

            mov ebx, cr0
            or ebx, 0x80000000
            mov cr0, ebx
        ", in(reg) PAGE_DIR_ADDRESS);
    }
}

pub fn init_identity() {
    let x: &mut [usize; 1024] = unsafe {
        core::mem::transmute::<usize, &mut [usize; 1024]>(PAGE_DIR_ADDRESS)
    };
    x.iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v = (FIRST_PAGE_TABLE + i*4*1024) | 3);

    let y: &mut [[usize; 1024]; 1024]  = unsafe {
        core::mem::transmute::<usize, &mut [[usize; 1024]; 1024]>(FIRST_PAGE_TABLE)
    };
    for i in 0..1024 {
        for j in 0..1024 {
            y[i][j] = ((i*1024 + j)*4096) | 3;
        }
    }

    translate(0x10000);
    translate(0x7b25);
    println!("{:#x}", (*y)[0][0]);
    println!("{:#x}", (*y)[0][3]);
    println!("{:#x}", y[0][7]);
    println!("{:#x}", y[0][42]);
    println!("{:#x}", y[0][1023]);

    /*
    for i in 0..0xFFFFFFFF {
        translate(i);
    }
    */
    enable_paging();
}

fn translate(add: u32) {
    let d_offset = add >> 22;
    let t_offset = (add & 0x3FF000) >> 12;
    let p_offset = add & 0xFFF;
    println!("{} {} {}", d_offset, t_offset, p_offset);
    unsafe {
        let d_entry: u32 = *((PAGE_DIR_ADDRESS as u32 + d_offset*4) as *const u32);
        let t_entry: u32 = *(((d_entry & 0xFFFFF000) + t_offset*4) as *const u32);
        let padd: u32 = (t_entry & 0xFFFFF000) + p_offset;
        println!("d_entry: {:#x} t_entry: {:#x}", d_entry, t_entry);
        println!("vadd {:#x} padd {:#x}", add, padd);
    }
}
