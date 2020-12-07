//! Kernel From Scratch
//!
//! This crate contains a simple kernel.
//!
//! # Features
//!
//! - Keyboard inputs are printed on screen

//#![warn(missing_docs)]
//#![warn(missing_doc_code_examples)]

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(associated_type_bounds)]
#![no_std]

use core::panic::PanicInfo;

extern crate spin;

#[macro_use]
pub mod writer;
pub mod gdt;
pub mod io_port;
pub mod keyboard;
pub mod ps2;
pub mod stack;

use keyboard::{Command, KEYBOARD};
use ps2::PS2;
use writer::WRITER;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("\nCRASHED");
    stack::trace(10);
    loop {}
}

fn init() {
    gdt::init();
    WRITER.lock().clear_screen();
    PS2.lock().init();
}

/// The kernel entry point.
///
/// This is the function called by grub after reading the multiboot header.
/// It first initializes hardwares and wait for keyboard inputs to display on
/// screen.
#[no_mangle]
pub extern "C" fn kernel_main() {
    init();
    loop {
        let c = PS2.lock().read();
        match KEYBOARD.lock().handle_scan_code(c as usize) {
            keyboard::Key::Character(c) if c != 0x0 as char => print!("{}", c),
            keyboard::Key::Command(Command::Left) => WRITER.lock().left(),
            keyboard::Key::Command(Command::Right) => WRITER.lock().right(),
            keyboard::Key::Command(Command::Prev) => WRITER.lock().prev_screen(),
            keyboard::Key::Command(Command::Next) => WRITER.lock().next_screen(),
            _ => (),
        }
    }
}
