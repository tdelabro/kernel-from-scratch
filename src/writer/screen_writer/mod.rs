mod screen_char;

use self::screen_char::{Color, ColorCode, ScreenChar};
use alloc::prelude::v1::Vec;
use alloc::{slice, vec};
use core::fmt;
use core::ptr::{read_volatile, write_volatile, Unique};

pub struct VGAScreen {
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<ScreenChar>,
    pub width: usize,
    pub height: usize,
}

impl fmt::Write for VGAScreen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

impl VGAScreen {
    pub const fn new(addr: usize, width: usize, height: usize) -> Self {
        VGAScreen {
            column_position: 0x0,
            color_code: ColorCode::new(Color::LightGreen, Color::Black),
            buffer: unsafe { Unique::new_unchecked(addr as *mut _) },
            width,
            height,
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
                if self.column_position >= self.width {
                    self.new_line();
                }
                let row = self.height - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                if self.get_screenchar(row, col).ascii_character != 0x0 {
                    self.shift_right();
                }
                self.set_screenchar(
                    row,
                    col,
                    ScreenChar {
                        ascii_character: byte,
                        color_code,
                    },
                );

                if self.column_position + 1 >= self.width {
                    self.new_line();
                } else {
                    self.change_position(self.column_position + 1);
                }
            }
        }
    }

    /// Full screen clear
    pub fn clear_screen(&mut self) {
        for _ in 0..self.height {
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
        if self.column_position < self.width - 1 {
            let col = self.column_position;
            let c = self.get_screenchar(self.height - 1, col).ascii_character;
            if c != 0x0 {
                self.change_position(self.column_position + 1);
            }
        }
    }

    fn unblink_current(&mut self) {
        let row = self.height - 1;
        let col = self.column_position;
        let mut character = self.get_screenchar(row, col);
        character.unblink();
        self.set_screenchar(row, col, character);
    }

    fn blink_current(&mut self) {
        let row = self.height - 1;
        let col = self.column_position;
        let mut character = self.get_screenchar(row, col);
        character.blink();
        self.set_screenchar(row, col, character);
    }

    fn change_position(&mut self, new_col: usize) {
        self.unblink_current();
        self.column_position = new_col;
        self.blink_current();
    }

    fn shift_right(&mut self) {
        let row = self.height - 1;
        let col = self.column_position;
        let mut end_line: usize = col;

        while end_line < self.width - 1 && self.get_screenchar(row, end_line).ascii_character != 0x0
        {
            end_line += 1;
        }
        while end_line > col {
            let character = self.get_screenchar(row, end_line - 1);
            self.set_screenchar(row, end_line, character);
            end_line -= 1;
        }
    }

    fn shift_left(&mut self) {
        let row = self.height - 1;
        let col = self.column_position;
        let mut i: usize = col;

        while i < self.width && self.get_screenchar(row, i).ascii_character != 0x0 {
            let character = self.get_screenchar(row, i);
            self.set_screenchar(row, i - 1, character);
            i += 1;
        }
        self.set_screenchar(
            row,
            i - 1,
            ScreenChar {
                ..Default::default()
            },
        );
        self.change_position(col - 1);
    }

    fn new_line(&mut self) {
        self.unblink_current();
        for row in 1..self.height {
            for col in 0..self.width {
                let character = self.get_screenchar(row, col);
                self.set_screenchar(row - 1, col, character);
            }
        }
        self.clear_row(self.height - 1);
        self.change_position(0);
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..self.width {
            self.set_screenchar(
                row,
                col,
                ScreenChar {
                    ..Default::default()
                },
            );
        }
    }

    fn get_screenchar(&self, row: usize, col: usize) -> ScreenChar {
        unsafe {
            let slc = slice::from_raw_parts(self.buffer.as_ptr(), self.width * self.height);
            read_volatile(&slc[row * self.width + col])
        }
    }

    fn set_screenchar(&mut self, row: usize, col: usize, character: ScreenChar) {
        unsafe {
            let slc = slice::from_raw_parts_mut(self.buffer.as_ptr(), self.width * self.height);
            write_volatile(&mut slc[row * self.width + col], character);
        }
    }

    /// Copy the last line in a buffer
    pub fn get_bottom_line(&self) -> Vec<u8> {
        let mut ascii_line = vec![0u8; self.width];

        for (i, sc) in ascii_line.iter_mut().enumerate() {
            *sc = self.get_screenchar(self.height - 1, i).ascii_character;
        }

        ascii_line
    }

    pub fn swap_bottom_line(&mut self, ascii_line: &[u8]) {
        let color_code = self.color_code;
        let mut end = 0;
        for (i, c) in ascii_line.iter().enumerate().take(self.width) {
            self.set_screenchar(
                self.height - 1,
                i,
                ScreenChar {
                    ascii_character: *c,
                    color_code,
                },
            );
            if *c != 0x0 {
                end = i;
            }
        }
        self.change_position(end + 1);
    }
}
