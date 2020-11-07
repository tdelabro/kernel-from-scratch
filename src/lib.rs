#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(llvm_asm)]
#![feature(associated_type_bounds)]
#![no_std]

use core::panic::PanicInfo;

extern crate spin;

#[macro_use]
mod vga_buffer;
mod keyboard;
mod io_port;
mod ps2;
mod pic;

use pic::PICS;
use vga_buffer::WRITER;
use keyboard::KEYBOARD;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    WRITER.lock().clear_screen();
    println!("Hello World{}", "!");
    ps2::PS2.lock().initialize();
    unsafe { PICS.lock().initialize(); }
    loop {
        let c = ps2::PS2.lock().read();
        match KEYBOARD.lock().handle_scan_code(c as usize) {
            Some(c) if c != 0x0 as char => print!("{}", c),
            _ => (),
        }
    }
}
