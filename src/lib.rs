//! Kernel From Scratch
//!
//! This crate contains tdelabro minimalist kernel, for the purpose of the school 42 kfs projects.
//!
//! Current step: kfs-3.
//!
//! # Features
//! - Global Descriptor Table
//! - VGA Screen
//! - Keyboard input
//! - Basic command line interpreter
//! - Debug utilities
//! - Power management
//! - Physical memory management
//! - Paging & virtual memory management
//! - Unique Kernel heap
//! - Multiple user heaps
//! - alloc crate compatibility
//! - Dynamic framebuffer and RAM size

//#![warn(missing_docs)]
//#![warn(missing_doc_code_examples)]

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(asm)]
#![feature(associated_type_bounds)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_ptr_offset)]
#![feature(alloc_error_handler)]
#![no_std]

extern crate alloc;
extern crate spin;

use core::panic::PanicInfo;

#[macro_use]
pub mod writer;
pub mod debug;
pub mod dynamic_memory_management;
pub mod external_symbols;
pub mod gdt;
pub mod heap_demo;
pub mod io_port;
pub mod keyboard;
pub mod multiboot_info;
pub mod physical_memory_management;
pub mod power_management;
pub mod ps2;
pub mod shell;
pub mod virtual_memory_management;

use keyboard::{Command, KEYBOARD};
use multiboot_info::MultibootInfo;
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

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn init(magic_number: usize, multiboot: MultibootInfo) {
    assert_eq!(
        magic_number, 0x36d76289,
        "System hadn't been loaded by a Multiboot2-compliant boot loader."
    );

    // Framebuffer
    writer::init(multiboot);

    let tag = multiboot.get_framebuffer().unwrap();
    println!("{} {}", tag.framebuffer_width, tag.framebuffer_height);

    // Global Descriptor Table
    gdt::init();

    // Paging
    virtual_memory_management::init(true, multiboot);

    // Keyboard input
    PS2.lock().init();
}

/// The kernel entry point.
///
/// This is the function called by grub after reading the multiboot header.
/// It first initializes hardwares and wait for keyboard inputs to display on
/// screen.
#[no_mangle]
pub extern "C" fn kernel_main(magic_number: usize, p_multiboot_info: MultibootInfo) {
    init(magic_number, p_multiboot_info);
    debug::print_kernel_sections_addresses();
    loop {
        let c = PS2.lock().read();
        match KEYBOARD.lock().handle_scan_code(c as usize) {
            keyboard::Key::Character(c) if c != 0x0 as char => print!("{}", c),
            keyboard::Key::Command(Command::Left) => WRITER.lock().as_mut().unwrap().left(),
            keyboard::Key::Command(Command::Right) => WRITER.lock().as_mut().unwrap().right(),
            keyboard::Key::Command(Command::Enter) => shell::execute(),
            keyboard::Key::Command(Command::LastCommand) => shell::load_last_command(),
            _ => (),
        }
    }
}
