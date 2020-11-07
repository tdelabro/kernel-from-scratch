#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    fn blink(&mut self) {
        self.color_code.0 = self.color_code.0 | (1 << 7);
    }

    fn unblink(&mut self) {
        self.color_code.0 = self.color_code.0 & !(1 << 7);
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

use core::ptr::Unique;

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>,
}

use core::ptr::read_volatile;
use core::ptr::write_volatile;

impl Writer {
    fn unblink_current(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut character = self.buffer().chars[row][col];
        character.unblink();
        unsafe {
            write_volatile(&mut self.buffer().chars[row][col], character);
        }
    }

    fn blink_current(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut character = self.buffer().chars[row][col];
        character.blink();
        unsafe {
            write_volatile(&mut self.buffer().chars[row][col], character);
        }
    }

    fn change_position(&mut self, new_col: usize) {
        self.unblink_current();
        self.column_position = new_col;
        self.blink_current();
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            0x09 => {
                self.write_byte(0x20);
                while self.column_position % 4 != 0 {
                    self.write_byte(0x20);
                }
            }
            0x0A => self.new_line(),
            0x0D => self.new_line(),
            0x8 => {
                if self.column_position > 0 {
                    self.shift_left();
                }
            },
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                if self.buffer().chars[row][col].ascii_character != 0x0 {
                    self.shift_right();
                }
                unsafe {
                    write_volatile(
                        &mut self.buffer().chars[row][col],
                        ScreenChar {
                            ascii_character: byte,
                            color_code: color_code,
                        },
                    );
                }
                if self.column_position + 1 >= BUFFER_WIDTH {
                    self.new_line();
                } else {
                    self.change_position(self.column_position + 1);
                }
            }
        }
    }

    fn shift_right(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut end_line: usize = col;
        let buffer = self.buffer();

        while buffer.chars[row][end_line].ascii_character != 0x0 {
            end_line += 1;
        }
        while end_line > col {
            unsafe {
                let character = read_volatile(&buffer.chars[row][end_line-1]);
                write_volatile(&mut buffer.chars[row][end_line], character);
            }
            end_line -= 1;
        }
    }

    fn shift_left(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut i: usize = col;
        let buffer = self.buffer();

        while buffer.chars[row][i].ascii_character != 0x0 {
            unsafe {
                let character = read_volatile(&buffer.chars[row][i]);
                write_volatile(&mut buffer.chars[row][i-1], character);
            }
            i += 1;
        }
        unsafe {
            let character = read_volatile(&buffer.chars[row][i]);
            write_volatile(&mut buffer.chars[row][i-1], character);
        }
        self.change_position(col - 1);
    }

    fn new_line(&mut self) {
        self.unblink_current(); 
        let buffer = self.buffer();
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    let character = read_volatile(&buffer.chars[row][col]);
                    write_volatile(&mut buffer.chars[row - 1][col], character);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.change_position(0);
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            let buffer = self.buffer();
            unsafe {
                write_volatile(
                    &mut buffer.chars[row][col],
                    ScreenChar {
                        ascii_character: 0,
                        color_code: ColorCode(0),
                    },
                );
            }
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    pub fn clear_screen(&mut self) {
        for _ in 0..BUFFER_HEIGHT {
            self.new_line();
        }
    }

    pub fn left(&mut self) {
        if self.column_position > 0 {
            self.change_position(self.column_position - 1);
        }
    }

    pub fn right(&mut self) {
        if self.column_position < BUFFER_WIDTH - 1 {
            let col = self.column_position;
            let c = self.buffer().chars[BUFFER_HEIGHT - 1][col].ascii_character; 
            if c != 0x0 {
                self.change_position(self.column_position + 1);
            }
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

use spin::Mutex;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
});

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga_buffer::print_args(format_args!($($arg)*));
    });
}

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

