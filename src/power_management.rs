//! Power Management tools

/// Shutdown the system
///
/// Use the BIOS interrupt 0x15.
pub fn shutdown() {
    unsafe {
        asm!("  mov ax, 0x5307
                mov bx, 0x0001
                mov cx, 0x0003
                int 0x15
                ret",
                out("ax") _, out("bx") _, out("cx") _);
    }
}

/// Reboot the system
///
/// Far jump to the reset vector in real mode, triple fault in protected mode.
pub fn reboot() {
    unsafe {
        asm!("ljmp $0xffff, $0", options(att_syntax));
    }
}
