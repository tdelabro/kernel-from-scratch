//! Minimal shell
//!
//! Handle a set of basic user instructions.

use crate::writer::{WRITER, BUFFER_WIDTH};
use core::str::SplitWhitespace;

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
            _ => (),
        };
}

fn dump(mut words: SplitWhitespace) {
        match words.next() {
            Some("seg_reg") => crate::debug::dump_segment_registers(),
            Some("gdtr") => crate::debug::dump_gdtr(),
            Some("stack") => crate::debug::dump_stack(get_number(words)),
            Some("trace") => crate::debug::stack_trace(get_number(words)),
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
