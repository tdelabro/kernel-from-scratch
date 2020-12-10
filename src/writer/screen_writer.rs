#[repr(u8)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

impl ScreenChar {
    fn blink(&mut self) {
        self.color_code.0 = (self.color_code.0 & !(0xF << 4)) | (0x8 << 4);
    }

    fn unblink(&mut self) {
        self.color_code.0 &= !(0xF << 4);
    }
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[derive(Clone, Copy)]
pub struct Buffer {
    pub chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Write characters on screen
///
/// Characters are written on the last line of the screen, according to a
/// specific color code.
#[derive(Clone, Copy)]
pub struct Screen {
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: Buffer,
}

use core::ptr::Unique;

const NUMBER_OF_SCREENS: usize = 6;
const DEFAULT_COLOR_CODE: ColorCode = ColorCode::new(Color::LightGreen, Color::Black);

pub struct ScreenWriter {
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>,
    pub screens: [Screen; NUMBER_OF_SCREENS],
    pub index: usize,
}

use core::ptr::read_volatile;
use core::ptr::write_volatile;

impl ScreenWriter {
    pub const fn new() -> ScreenWriter {
        ScreenWriter {
            column_position: 0,
            color_code: DEFAULT_COLOR_CODE,
            buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
            screens: [Screen {
                column_position: 0,
                color_code: DEFAULT_COLOR_CODE,
                buffer: Buffer {
                    chars: [[ScreenChar {
                        ascii_character: 0x0,
                        color_code: DEFAULT_COLOR_CODE,
                    }; BUFFER_WIDTH]; BUFFER_HEIGHT],
                },
            }; NUMBER_OF_SCREENS],
            index: 0,
        }
    }

    /// Dispaly character on the screen
    ///
    /// Characters are written on the bottom line of the screen.
    ///
    /// # Features
    ///
    /// - New line
    /// - Tabulation
    /// - Back space
    /// - Moving cursor left and right
    /// - Insert characters at any position on the line
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            // Tabulation
            0x09 => {
                self.write_byte(0x20);
                while self.column_position % 4 != 0 {
                    self.write_byte(0x20);
                }
            }
            // Line feed and Cariage return
            0x0A | 0x0D => self.new_line(),
            // Backspace
            0x8 => {
                if self.column_position > 0 {
                    self.shift_left();
                }
            }
            // Other Ascii characters
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                if self.mut_buffer().chars[row][col].ascii_character != 0x0 {
                    self.shift_right();
                }
                unsafe {
                    write_volatile(
                        &mut self.mut_buffer().chars[row][col],
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

    /// Full screen clear
    pub fn clear_screen(&mut self) {
        for _ in 0..BUFFER_HEIGHT {
            self.new_line();
        }
    }

    /// Move cursor to the left
    pub fn left(&mut self) {
        if self.column_position > 0 {
            self.change_position(self.column_position - 1);
        }
    }
    /// Move cursor to the right
    pub fn right(&mut self) {
        if self.column_position < BUFFER_WIDTH - 1 {
            let col = self.column_position;
            let c = self.mut_buffer().chars[BUFFER_HEIGHT - 1][col].ascii_character;
            if c != 0x0 {
                self.change_position(self.column_position + 1);
            }
        }
    }

    fn unblink_current(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut character = self.mut_buffer().chars[row][col];
        character.unblink();
        unsafe {
            write_volatile(&mut self.mut_buffer().chars[row][col], character);
        }
    }

    fn blink_current(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut character = self.mut_buffer().chars[row][col];
        character.blink();
        unsafe {
            write_volatile(&mut self.mut_buffer().chars[row][col], character);
        }
    }

    fn change_position(&mut self, new_col: usize) {
        self.unblink_current();
        self.column_position = new_col;
        self.blink_current();
    }

    fn shift_right(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut end_line: usize = col;
        let buffer = self.mut_buffer();

        while buffer.chars[row][end_line].ascii_character != 0x0 {
            end_line += 1;
        }
        while end_line > col {
            unsafe {
                let character = read_volatile(&buffer.chars[row][end_line - 1]);
                write_volatile(&mut buffer.chars[row][end_line], character);
            }
            end_line -= 1;
        }
    }

    fn shift_left(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut i: usize = col;
        let buffer = self.mut_buffer();

        while buffer.chars[row][i].ascii_character != 0x0 {
            unsafe {
                let character = read_volatile(&buffer.chars[row][i]);
                write_volatile(&mut buffer.chars[row][i - 1], character);
            }
            i += 1;
        }
        unsafe {
            let character = read_volatile(&buffer.chars[row][i]);
            write_volatile(&mut buffer.chars[row][i - 1], character);
        }
        self.change_position(col - 1);
    }

    fn new_line(&mut self) {
        self.unblink_current();
        let buffer = self.mut_buffer();
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
        let buffer = self.mut_buffer();
        for col in 0..BUFFER_WIDTH {
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

    fn mut_buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    fn ref_buffer(&self) -> &Buffer {
        unsafe { self.buffer.as_ref() }
    }

    fn load_screen(&mut self, index: usize) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    self.screens[self.index].buffer.chars[row][col] =
                        read_volatile(&self.ref_buffer().chars[row][col]);
                    write_volatile(
                        &mut self.mut_buffer().chars[row][col],
                        self.screens[index].buffer.chars[row][col],
                    );
                }
            }
        }
        self.screens[self.index].column_position = self.column_position;
        self.column_position = self.screens[index].column_position;
        self.index = index;
        self.blink_current();
    }

    /// Load an arbitrary screen
    pub fn swap_screen(&mut self, index: usize) {
        if index < NUMBER_OF_SCREENS {
            self.load_screen(index);
        }
    }

    /// Load next screen
    pub fn next_screen(&mut self) {
        self.load_screen(match self.index {
            i if i == NUMBER_OF_SCREENS - 1 => 0,
            _ => self.index + 1,
        });
    }

    /// Load previous screen
    pub fn prev_screen(&mut self) {
        self.load_screen(match self.index {
            0 => NUMBER_OF_SCREENS - 1,
            _ => self.index - 1,
        });
    }

    
    /// Copy the last line in a buffer
    pub fn get_bottom_line(&self, ascii_line: &mut [u8; BUFFER_WIDTH]) {
        let buffer = self.ref_buffer();
        for i in 0..BUFFER_WIDTH {
            ascii_line[i] = unsafe { 
                read_volatile(&buffer.chars[BUFFER_HEIGHT - 1][i]).ascii_character
            };
        }
    }
}

use core::fmt;

impl fmt::Write for ScreenWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}
