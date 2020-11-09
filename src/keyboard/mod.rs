//! Keyboard driver
//!
//! # Features
//! - Set 2 scan codes
//! - Shift key
//! - Left and right arrow

mod sets;

struct KeyMap {
    key_array: [char; 0x84],
    shift_key_array: [char; 0x84],
}

struct State(u8);

impl State {
    fn set_release(&mut self) {
        self.0 |= 1;
    }

    fn clear_release(&mut self) {
        self.0 &= !(1);
    }

    fn get_release(&self) -> bool {
        self.0 & 1 != 0
    }

    fn refresh_shift(&mut self) {
        if self.get_release() {
            self.0 &= !(1 << 1);
            self.clear_release();
        } else {
            self.0 |= 1 << 1;
        }
    }

    fn get_shift(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    fn set_extended(&mut self) {
        self.0 |= 1 << 2;
    }

    fn clear_extended(&mut self) {
        self.0 &= !(1 << 2);
    }

    fn get_extended(&self) -> bool {
        self.0 & (1 << 2) != 0
    }
}

/// Scan code interpretation
///
/// Can either be a Character (to print), a Command (to perform) or a NOP.
pub enum Key {
    Character(char),
    Command(usize),
    None,
}

/// Scan code interpreter
///
/// Interpret scan codes according to SC/2 configuration.
/// 
/// Internal state is updated according to recieved scan codes and may change
/// the way next scan codes are interpreted.
pub struct Keyboard {
    state: State,
    keymap: KeyMap,
}

impl Keyboard {
    /// Parse scan code into key
    ///
    /// # Special scan codes
    ///
    /// - 0xE0 signal extended key
    /// - 0xF0 signal released key
    pub fn handle_scan_code(&mut self, scan_code: usize) -> Key {
	match scan_code {
	    0xE0 => {
		self.state.set_extended();
		Key::None
	    }
	    0x59 | 0x12 => {
		self.state.refresh_shift();
		Key::None
	    }
	    0xF0 => {
		self.state.set_release();
		Key::None
	    }
	    sc if self.state.get_extended() => {
		self.state.clear_extended();
		if self.state.get_release() {
		    self.state.clear_release();
		    Key::None
		} else {
		    Key::Command(sc)
		}
	    }
	    sc if sc <= 0x83 => {
		if self.state.get_release() {
		    self.state.clear_release();
		    Key::None
		} else {
		    Key::Character(self.get_char(scan_code))
		}
	    }
	    _ => Key::None,
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

/// Keyboard
pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    keymap: KeyMap {
	key_array: sets::SET_2,
	shift_key_array: sets::SET_2_SHIFT,
    },
    state: State(0),
});
