#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![no_std]

use core::panic::PanicInfo;

extern crate spin;

#[macro_use]
mod vga_buffer;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    vga_buffer::clear_screen();
    println!("Hello World{}", "!");
}
