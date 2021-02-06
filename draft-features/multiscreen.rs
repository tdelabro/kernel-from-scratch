const NUMBER_OF_SCREENS: usize = 6;

struct ScreenWriter {
    pub screens: [Screen; NUMBER_OF_SCREENS],
    pub screen_index: usize,
}

impl ScreenWriter {
    pub const fn new() -> ScreenWriter {
        ScreenWriter {
            screens: [Default::default(); NUMBER_OF_SCREENS],
            ..Default::default()
        }
    }

    fn load_screen(&mut self, index: usize) {
        for r in 0..BUFFER_HEIGHT {
            for c in 0..BUFFER_WIDTH {
                unsafe {
                    self.screens[self.screen_index].buffer[r][c] = read_volatile(&self.ref_buffer()[r][c]);
                    write_volatile(&mut self.mut_buffer()[r][c], self.screens[index].buffer[r][c]);
                }
            }
        }
        self.screens[self.screen_index].column_position = self.column_position;
        self.column_position = self.screens[index].column_position;
        self.screen_index = index;
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
        self.load_screen(match self.screen_index {
            i if i == NUMBER_OF_SCREENS - 1 => 0,
            _ => self.screen_index + 1,
        });
    }

    /// Load previous screen
    pub fn prev_screen(&mut self) {
        self.load_screen(match self.screen_index {
            0 => NUMBER_OF_SCREENS - 1,
            _ => self.screen_index - 1,
        });
    }
}

