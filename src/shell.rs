//! Minimal shell
//!
//! Handle a set of basic user instructions.

use crate::writer::{WRITER, BUFFER_WIDTH};
use core::str::SplitWhitespace;

use spin::Mutex;

pub static LAST_COMMAND: Mutex<[u8; BUFFER_WIDTH]> = Mutex::new(
    [0u8; BUFFER_WIDTH]);

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
    let mut ascii_line = [0x0u8; BUFFER_WIDTH];
    WRITER.lock().get_bottom_line(&mut ascii_line);
    println!("");

    let mut words = match core::str::from_utf8(&ascii_line) {
        Ok(s) => s.trim_matches(0x0 as char).trim().split_whitespace(),
        Err(_) => { return; },
    };
    match words.next() {
        Some("dump") => dump(words),
        Some("shutdown") => crate::power_management::shutdown(),
        Some("reboot") => crate::power_management::reboot(),
        Some("clear") => WRITER.lock().clear_screen(),
        _ => (),
    };

    for i in 0..BUFFER_WIDTH {
        LAST_COMMAND.lock()[i] = ascii_line[i];
    }
}

/// Load last command
///
/// Replace the current terminal line by the last command that have been
/// executed
pub fn load_last_command() {
    WRITER.lock().swap_bottom_line(&LAST_COMMAND.lock());
}

fn dump(mut words: SplitWhitespace) {
    match words.next() {
        Some("seg_reg") => crate::debug::dump_segment_registers(),
        Some("gdtr") => crate::debug::dump_gdtr(),
        Some("gdt") => crate::debug::dump_gdt(),
        Some("stack") => crate::debug::dump_stack(get_number(words)),
        Some("trace") => crate::debug::stack_trace(get_number(words)),
        Some("bitmap") => crate::debug::dump_bitmap(),
        _ => (),
    };
}

fn get_number(mut words: SplitWhitespace) -> usize {
    match words.next() {
        Some(s) => match s.parse() {
            Ok(n) => n,
            Err(_) => 0,
        },
        _ => 0,
    }
}
