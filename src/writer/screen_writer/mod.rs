mod screen_char;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
pub const VGA_ADDRESS: usize = 0xb8000;

use core::ptr::Unique;
use self::screen_char::{ScreenChar, ColorCode, Color};

pub struct VGAScreen {
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<[[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]>,
}

use core::fmt;

impl fmt::Write for VGAScreen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

use core::ptr::read_volatile;
use core::ptr::write_volatile;

impl VGAScreen {
    pub const fn new() -> Self {
        VGAScreen {
            column_position: 0x0,
            color_code: ColorCode::new(Color::LightGreen, Color::Black),
            buffer: unsafe { Unique::new_unchecked(VGA_ADDRESS as *mut _) },
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
                if self.ref_buffer()[row][col].ascii_character != 0x0 {
                    self.shift_right();
                }
                unsafe {
                    write_volatile(
                        &mut self.mut_buffer()[row][col],
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
            let c = self.ref_buffer()[BUFFER_HEIGHT - 1][col].ascii_character;
            if c != 0x0 {
                self.change_position(self.column_position + 1);
            }
        }
    }

    fn unblink_current(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut character = self.ref_buffer()[row][col];
        character.unblink();
        unsafe {
            write_volatile(&mut self.mut_buffer()[row][col], character);
        }
    }

    fn blink_current(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let mut character = self.ref_buffer()[row][col];
        character.blink();
        unsafe {
            write_volatile(&mut self.mut_buffer()[row][col], character);
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

        while end_line < BUFFER_WIDTH - 1 && buffer[row][end_line].ascii_character != 0x0 {
            end_line += 1;
        }
        while end_line > col {
            unsafe {
                let character = read_volatile(&buffer[row][end_line - 1]);
                write_volatile(&mut buffer[row][end_line], character);
            }
            end_line -= 1;
        }
    }

    fn shift_left(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let buffer = self.mut_buffer();
        let mut i: usize = col;

        while i < BUFFER_WIDTH && buffer[row][i].ascii_character != 0x0 {
            unsafe {
                let character = read_volatile(&buffer[row][i]);
                write_volatile(&mut buffer[row][i - 1], character);
            }
            i += 1;
        }
        unsafe {
            write_volatile(&mut buffer[row][i-1], ScreenChar { ..Default::default() });
        }
        self.change_position(col - 1);
    }

    fn new_line(&mut self) {
        self.unblink_current();
        let buffer = self.mut_buffer();
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    let character = read_volatile(&buffer[row][col]);
                    write_volatile(&mut buffer[row - 1][col], character);
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
                write_volatile(&mut buffer[row][col], ScreenChar { ..Default::default() });
            }
        }
    }

    fn mut_buffer(&mut self) -> &mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT] {
        unsafe { self.buffer.as_mut() }
    }

    fn ref_buffer(&self) -> &[[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT] {
        unsafe { self.buffer.as_ref() }
    }


    /// Copy the last line in a buffer
    pub fn get_bottom_line(&self, ascii_line: &mut [u8; BUFFER_WIDTH]) {
        let buffer = self.ref_buffer();
        for i in 0..BUFFER_WIDTH {
            ascii_line[i] = unsafe { 
                read_volatile(&buffer[BUFFER_HEIGHT - 1][i]).ascii_character
            };
        }
    }

    pub fn swap_bottom_line(&mut self, ascii_line: &[u8; BUFFER_WIDTH]) {
        let color_code = self.color_code;
        let buffer = self.mut_buffer();
        let mut end = 0;
        for i in 0..BUFFER_WIDTH {
            unsafe {
                write_volatile(&mut buffer[BUFFER_HEIGHT - 1][i], ScreenChar {
                        ascii_character: ascii_line[i],
                        color_code: color_code,
                });
            }
            if ascii_line[i] != 0x0 {
                end = i;
            }
        }
        self.change_position(end + 1);
    }
}
