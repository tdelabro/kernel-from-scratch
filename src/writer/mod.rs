//! Screen driver
//!
//! # Features
//!
//! - Print characters on the bottom line of the screen
//! - A new line is added when the current is full
//! - The cursor can be moved along the line to write at specific position
//! - Characters can be removed from screen with backspace
//! - 6 screens exist. Loop between them with ctrl + left/right arrow

mod screen_writer;

use spin::Mutex;

pub use self::screen_writer::BUFFER_WIDTH;
use self::screen_writer::VGAScreen;

/// The unique entry point to write characters on the VGA screen
pub static WRITER: Mutex<VGAScreen> = Mutex::new(VGAScreen::new());

use core::fmt;

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
	$crate::writer::print_args(format_args!($($arg)*));
    });
}
