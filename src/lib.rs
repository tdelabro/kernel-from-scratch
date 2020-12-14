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

use keyboard::{Command, KEYBOARD};
use ps2::PS2;
use writer::WRITER;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("\nPANIC");
    println!("Stack Trace:");
    debug::stack_trace(20);
    loop {}
}

fn init() {
    gdt::init();
    WRITER.lock().clear_screen();
    PS2.lock().init();
    WRITER.lock().clear_screen();
}

/// Symbols defined in the linker script
extern "C" {
    fn kernel_memory_end();
    fn kernel_memory_start();
    fn kernel_memory_text();
    fn kernel_memory_rodata();
    fn kernel_memory_data();
    fn kernel_memory_bss();
}

fn get_linked_symbol_address(f: unsafe extern "C" fn()) -> u32 {
    f as *const u32 as u32
}

/// The kernel entry point.
///
/// This is the function called by grub after reading the multiboot header.
/// It first initializes hardwares and wait for keyboard inputs to display on
/// screen.
#[no_mangle]
pub extern "C" fn kernel_main() {
    init();
    paging::PAGE_DIRECTORY.lock().init_identity();
    paging::enable();
    println!("kmemend {:#x} {:#x} {:#x} {:#x} {:#x} {:#x}",
        get_linked_symbol_address(kernel_memory_start),
        get_linked_symbol_address(kernel_memory_text),
        get_linked_symbol_address(kernel_memory_rodata),
        get_linked_symbol_address(kernel_memory_data),
        get_linked_symbol_address(kernel_memory_bss),
        get_linked_symbol_address(kernel_memory_end));
    loop {
        let c = PS2.lock().read();
        match KEYBOARD.lock().handle_scan_code(c as usize) {
            keyboard::Key::Character(c) if c != 0x0 as char => print!("{}", c),
            keyboard::Key::Command(Command::Left) => WRITER.lock().left(),
            keyboard::Key::Command(Command::Right) => WRITER.lock().right(),
            keyboard::Key::Command(Command::Prev) => WRITER.lock().prev_screen(),
            keyboard::Key::Command(Command::Next) => WRITER.lock().next_screen(),
            keyboard::Key::Command(Command::Enter) => shell::execute(),
            keyboard::Key::Command(Command::LastCommand) => shell::load_last_command(),
            _ => (),
        }
    }
}
