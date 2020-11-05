#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(llvm_asm)]
#![no_std]

use core::panic::PanicInfo;

extern crate spin;

#[macro_use]
mod vga_buffer;
mod io_port;
mod keyboard;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    vga_buffer::clear_screen();
    println!("Hello World{}", "!");
    keyboard::init_ps2();

}
