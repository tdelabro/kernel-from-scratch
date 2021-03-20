//! Screen driver
//!
//! # Features
//!
//! - Print characters on the bottom line of the screen
//! - A new line is added when the current is full
//! - The cursor can be moved along the line to write at specific position
//! - Characters can be removed from screen with backspace

mod screen_writer;

use spin::Mutex;

use self::screen_writer::VGAScreen;
use core::convert::TryInto;
use MultibootInfo;

/// Unique enty point to write characters on the VGA screen
pub static WRITER: Mutex<Option<VGAScreen>> = Mutex::new(None);

use core::fmt;

pub fn init(multiboot: MultibootInfo) {
    let tag = multiboot.get_framebuffer().unwrap();

    WRITER.lock().replace(VGAScreen::new(
        tag.framebuffer_addr.try_into().unwrap(),
        tag.framebuffer_width.try_into().unwrap(),
        tag.framebuffer_height.try_into().unwrap(),
    ));
    WRITER.lock().as_mut().unwrap().clear_screen();
}

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
	$crate::writer::print_args(format_args!($($arg)*));
    });
}
