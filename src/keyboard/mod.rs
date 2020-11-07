mod set_2;

use crate::vga_buffer::WRITER;

struct KeyMap {
    key_array: [char; 0x84],
    shift_key_array: [char; 0x84],
}

struct State(u8);

impl State {
    pub fn set_release(&mut self) {
        self.0 |= 1;
    }

    pub fn clear_release(&mut self) {
        self.0 &= !(1);
    }

    pub fn get_release(&self) -> bool {
        self.0 & 1 != 0
    }

    pub fn refresh_shift(&mut self) {
        if self.get_release() {
            self.0 &= !(1 << 1);
            self.clear_release();
        } else {
            self.0 |= 1 << 1;
        }
    }

    pub fn get_shift(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub fn set_extended(&mut self) {
        self.0 |= 1 << 2;
    }

    pub fn clear_extended(&mut self) {
        self.0 &= !(1 << 2);
    }

    pub fn get_extended (&self) -> bool {
        self.0 & (1 << 2) != 0
    }
}

pub struct Keyboard {
    state: State,
    keymap: KeyMap,
}

impl Keyboard {
    pub fn handle_scan_code(&mut self, scan_code: usize) -> Option<char> {
        match scan_code {
            0xE0 => {
                self.state.set_extended();
                None
            },
            0x59 | 0x12 => {
                self.state.refresh_shift();
                None
            },
            0xF0 => { 
                self.state.set_release();
                None
            },
            sc if self.state.get_extended() => {
                self.state.clear_extended();
                if self.state.get_release() {
                    self.state.clear_release();
                } else {
                    match sc {
                        0x6B => WRITER.lock().left(),
                        0x74 => WRITER.lock().right(),
                        _ => (),
                    }
                }
                None
            },
            sc if sc <= 0x83  => {
                if self.state.get_release() {
                    self.state.clear_release();
                    None
                } else {
                    Some(self.get_char(scan_code))
                }
            },
            _ => None,
        }
    }

    fn get_char(&self, scan_code: usize) -> char {
        if !self.state.get_shift() {
            self.keymap.key_array[scan_code]
        } else {
            self.keymap.shift_key_array[scan_code]
        }
    }
}

use spin::Mutex;

pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    keymap: KeyMap {
        key_array: set_2::SET_2,
        shift_key_array: set_2::SET_2_SHIFT,
    },
    state: State(0),
});
