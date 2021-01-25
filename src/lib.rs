//! Kernel From Scratch
//!
//! This crate contains a simple kernel.
//!
//! # Features
//!
//! - Global Descriptor Table
//! - VGA Screen
//! - Keyboard input
//! - Basic command line interpreter
//! - Debug utilities
//! - Power management

//#![warn(missing_docs)]
//#![warn(missing_doc_code_examples)]

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
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
pub mod debug;
pub mod shell;
pub mod power_management;
pub mod page_frame;
pub mod paging;
pub mod external_symbols;

use keyboard::{Command, KEYBOARD};
use ps2::PS2;
use writer::WRITER;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    println!("Stack Trace:");
    debug::stack_trace(20);
    loop {}
}

fn init() {
    gdt::init();
    PS2.lock().init();
    paging::init();
}

/// The kernel entry point.
///
/// This is the function called by grub after reading the multiboot header.
/// It first initializes hardwares and wait for keyboard inputs to display on
/// screen.
#[no_mangle]
pub extern "C" fn kernel_main() {
    init();
    debug::print_kernel_sections_addresses();
    paging::list_mappings();
    loop {
        let c = PS2.lock().read();
        match KEYBOARD.lock().handle_scan_code(c as usize) {
            keyboard::Key::Character(c) if c != 0x0 as char => print!("{}", c),
            keyboard::Key::Command(Command::Left) => WRITER.lock().left(),
            keyboard::Key::Command(Command::Right) => WRITER.lock().right(),
            keyboard::Key::Command(Command::Enter) => shell::execute(),
            _ => (),
        }
    }
}
