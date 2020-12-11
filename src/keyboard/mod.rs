//! Keyboard driver
//!
//! # Features
//! - Shift key
//! - Control key
//! - Left and right arrow

mod sets;

struct KeyMap {
    key_array: [char; 0x84],
    shift_key_array: [char; 0x84],
}

struct State(u8);

impl State {
    // Release
    fn release(&self) -> bool {
        self.0 & 1 != 0
    }
    fn set_release(&mut self, value: bool) {
        if value {
            self.0 |= 1;
        } else {
            self.0 &= !(1);
        }
    }

    // Extended
    fn extended(&self) -> bool {
        self.0 & (1 << 1) != 0
    }
    fn set_extended(&mut self, value: bool) {
        if value {
            self.0 |= 1 << 1;
        } else {
            self.0 &= !(1 << 1);
        }
    }

    // Shift
    fn shift(&self) -> bool {
        self.0 & (1 << 2) != 0
    }
    fn set_shift(&mut self, value: bool) {
        if value {
            self.0 |= 1 << 2;
        } else {
            self.0 &= !(1 << 2);
        }
    }

    // Control
    fn control(&self) -> bool {
        self.0 & (1 << 3) != 0
    }
    fn set_control(&mut self, value: bool) {
        if value {
            self.0 |= 1 << 3;
        } else {
            self.0 &= !(1 << 3);
        }
    }

    fn refresh_shift(&mut self) {
        if self.release() {
            self.set_shift(false);
            self.set_release(false);
        } else {
            self.set_shift(true);
        }
    }

    fn refresh_control(&mut self) {
        self.set_extended(false);
        if self.release() {
            self.set_control(false);
            self.set_release(false);
        } else {
            self.set_control(true);
        }
    }
}

/// Specific instruction to perform
pub enum Command {
    /// Moove the screen cursor left
    Left = 0,
    /// Moove the screen cursor right
    Right = 1,
    /// Display previous screen
    Prev = 2,
    /// Display next screen
    Next = 3,
    /// Execute the current line
    Enter = 4,
    /// Load the last command executed
    LastCommand = 5,
}

/// Scan code interpretation
pub enum Key {
    /// An ascii character to be printed on screen
    Character(char),
    /// A command to execute
    Command(Command),
    /// NOP
    None,
}

/// Scan code interpreter
///
/// Handle scan codes according to it's keymap and state. Output the
/// character to print or the action to execute.
///
/// It's state evolve with the user input, modifying the way next scan codes
/// will be interpreted.
pub struct Keyboard {
    state: State,
    keymap: KeyMap,
}

impl Keyboard {
    /// Parse scan code into instruction
    ///
    /// # Special scan codes
    ///
    /// - 0xE0 signal extended key
    /// - 0xF0 signal released key
    pub fn handle_scan_code(&mut self, scan_code: usize) -> Key {
        match scan_code {
            // Release code
            0xF0 => {
                self.state.set_release(true);
                Key::None
            }
            // Extended code
            0xE0 => {
                self.state.set_extended(true);
                Key::None
            }
            // Shift codes (left and right keys)
            0x59 | 0x12 => {
                self.state.refresh_shift();
                Key::None
            }
            // Control codes (left and right keys)
            0x14 => {
                self.state.refresh_control();
                Key::None
            }
            // Interpreter when extended state is set
            sc if self.state.extended() => {
                self.state.set_extended(false);
                match sc {
                    _ if self.state.release() => {
                        self.state.set_release(false);
                        Key::None
                    }
                    0x6B if self.state.control() => Key::Command(Command::Prev),
                    0x74 if self.state.control() => Key::Command(Command::Next),
                    0x6B => Key::Command(Command::Left),
                    0x74 => Key::Command(Command::Right),
                    0x75 => Key::Command(Command::LastCommand),
                    _ => Key::None,
                }
            }
            // Interpreter for regular (non-extended) scan codes
            // 0x83 is their highest possible value in set 2
            sc if sc <= 0x83 => {
                if self.state.release() {
                    self.state.set_release(false);
                    Key::None
                } else {
                    match sc {
                        0x5A => Key::Command(Command::Enter),
                        _ => Key::Character(self.get_char(sc)),
                    }
                }
            }
            _ => Key::None,
        }
    }

    fn get_char(&self, scan_code: usize) -> char {
        if !self.state.shift() {
            self.keymap.key_array[scan_code]
        } else {
            self.keymap.shift_key_array[scan_code]
        }
    }
}

use spin::Mutex;

/// Set 2 keyboard interpreter
pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    keymap: KeyMap {
        key_array: sets::SET_2,
        shift_key_array: sets::SET_2_SHIFT,
    },
    state: State(0),
});
