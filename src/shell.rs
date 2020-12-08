struct CommandLine<'a> {
    raw: &'a str,
}

/*
impl CommandLine {
    pub fn new(s: &str) -> CommandLine {
        CommandLine {
            raw: s.trim(),
        }
    }
}
*/

use crate::writer::{WRITER, BUFFER_WIDTH};

pub fn execute() {
        let mut ascii_line = [0x0u8; BUFFER_WIDTH];
        WRITER.lock().get_current_line(&mut ascii_line);
        let input = match core::str::from_utf8(&ascii_line) {
            Ok(s) => s.trim_matches(0x0 as char).trim(),
            Err(_) => "",
        };
        println!("");
        match input {
            "dsr" => crate::debug::dump_segment_registers(),
            _ => (),
        };
}
