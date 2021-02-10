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
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_ptr_offset)]
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
pub mod physical_memory_management;
pub mod virtual_memory_management;
pub mod dynamic_memory_management;
pub mod external_symbols;

use keyboard::{Command, KEYBOARD};
use ps2::PS2;
use writer::WRITER;
use dynamic_memory_management::{KERNEL_HEAP, Box, Heap, Locked};
use core::alloc::Layout;
use core::alloc::GlobalAlloc;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    println!("Stack Trace:");
    debug::stack_trace(20);
    loop {}
}

fn init() {
    // Global Descriptor Table
    gdt::init();

    // Paging
    virtual_memory_management::init();

    // Keyboard input
    PS2.lock().init();
}

fn test_heap() {
    let heap = unsafe { Locked::new(Heap::new(0x181000 as *const _, false)) };
    let x = Box::new_in(5, &heap);
    println!("{:p} {}", x, x);
    let y = Box::new(12);
    println!("{:p} {}", y, y);
    unsafe {
    println!("1\n{}", heap.lock());
        let z = heap.alloc(Layout::new::<usize>());
    println!("2\n{}", heap.lock());
        *z = 4;
        println!("{:p} {}", z, *z);
        *z = 3;
        println!("{:p} {}", z, *z);
    heap.dealloc(z, Layout::new::<usize>());
    }
    println!("3\n{}", heap.lock());
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
    test_heap();
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
