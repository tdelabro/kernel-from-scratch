//! Minimal shell
//!
//! Handle a set of basic user instructions.

use crate::debug;
use crate::power_management;
use crate::writer::WRITER;
use core::str::SplitWhitespace;

use alloc::prelude::v1::Vec;
use spin::Mutex;

static LAST_COMMAND: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// Execute an user shell command
///
/// When Carriage Return is written, try to execute the current line.
///
/// # Valid instructions
/// - shutdown
/// - reboot
/// - dump
///     - seg_reg
///     - gdtr
///     - stack \[max\]
///     - trace \[max\]
///
pub fn execute() {
    let ascii_line = WRITER.lock().as_ref().unwrap().get_bottom_line();
    println!();

    let mut words = match core::str::from_utf8(&ascii_line) {
        Ok(s) => s.trim_matches(0x0 as char).trim().split_whitespace(),
        Err(_) => {
            return;
        }
    };
    match words.next() {
        Some("dump") => dump(words),
        Some("shutdown") => power_management::shutdown(),
        Some("reboot") => power_management::reboot(),
        Some("clear") => WRITER.lock().as_mut().unwrap().clear_screen(),
        _ => (),
    };

    LAST_COMMAND.lock().replace(ascii_line);
}

/// Load last command
///
/// Replace the current terminal line by the last command that have been
/// executed
pub fn load_last_command() {
    if let Some(cmd) = &*LAST_COMMAND.lock() {
        WRITER.lock().as_mut().unwrap().swap_bottom_line(cmd);
    }
}

fn dump(mut words: SplitWhitespace) {
    match words.next() {
        Some("seg_reg") => debug::dump_segment_registers(),
        Some("gdtr") => debug::dump_gdtr(),
        Some("gdt") => debug::dump_gdt(),
        Some("stack") => debug::dump_stack(get_number(words)),
        Some("trace") => debug::stack_trace(get_number(words)),
        Some("bitmap") => debug::dump_bitmap(),
        _ => (),
    };
}

fn get_number(mut words: SplitWhitespace) -> usize {
    match words.next() {
        Some(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}
