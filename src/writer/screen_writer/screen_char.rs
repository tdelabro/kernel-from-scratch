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

impl Default for ColorCode {
    fn default() -> Self {
        ColorCode::new(Color::LightGreen, Color::Black)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

impl ScreenChar {
    pub fn blink(&mut self) {
        self.color_code.0 = (self.color_code.0 & !(0xF << 4)) | (0x8 << 4);
    }

    pub fn unblink(&mut self) {
        self.color_code.0 &= !(0xF << 4);
    }
}
