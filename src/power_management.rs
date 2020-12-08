pub fn shutdown() {
    unsafe { asm!("
        mov ax, 0x5307
        mov bx, 0x0001
        mov cx, 0x0003
        int 0x15
        ret
        "); }
}

pub fn reboot() {
    unsafe { asm!("ljmp $0xffff, $0", options(att_syntax)); }
}
