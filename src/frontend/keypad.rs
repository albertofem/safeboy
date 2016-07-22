/// Keypad
///
/// They keypad is a simple hardware chip which contains
/// two rows of data (8 bits each) which contained
/// status for each key (one row for pressed, another row for releases).
pub struct Keypad {
    /// Data from key pressed
    ///
    /// Each key was represented as a 8bit value.
    /// Because we have two states (pressed, released), two
    /// registers of 8bits are used to store them
    keys: [u8; 2],

    /// Column to key states
    ///
    /// This represents which key state type (pressed, released)
    /// is selected to read/write at the moment
    column: u8,

    /// Keypad interrupt
    ///
    /// Indicates whether a key was pressed
    pub interrupt: u8
}

/// Keys
///
/// Enum containing all possible keys in the
/// GameBoy hardware
pub enum Key {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Right,
    Left
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keys: [
                0x0F,
                0x0F
            ],
            column: 0,
            interrupt: 0
        }
    }

    /// Read byte from the keypad
    ///
    /// Depending of what value we have in the
    /// column, we will read one state (pressed) or the other (released)
    pub fn read_byte(&self) -> u8 {
        match self.column {
            0x00 => {
                0x00
            },

            0x10 => {
                self.keys[0]
            },

            0x20 => {
                self.keys[1]
            },

            _ => panic!("Invalid keypad read")
        }
    }

    /// Write the column
    ///
    /// This is the only keypad write operation, to change
    /// which kind of keypress we are reading later
    pub fn write_byte(&mut self, value: u8) {
        self.column = value & 0x30;
    }

    /// Handles the key down
    ///
    /// Sets the corresponding keys values for the matching
    /// key pressed. This write of the keypad registers isn't
    /// handled by the MMU, but instead it's connected directly
    /// to the CPU (as it is user-driven I/O)
    pub fn key_down(&mut self, key: Key) {
        match key {
            Key::Right  => { self.keys[1] &= 0xE },
            Key::Left   => { self.keys[1] &= 0xD },
            Key::Up     => { self.keys[1] &= 0xB },
            Key::Down   => { self.keys[1] &= 0x7 },
            Key::A      => { self.keys[0] &= 0xE },
            Key::B      => { self.keys[0] &= 0xD },
            Key::Select => { self.keys[0] &= 0xB },
            Key::Start  => { self.keys[0] &= 0x7 },
        }

        self.interrupt |= 0x10;
    }

    /// Handles key releases
    ///
    /// Does the same as the key press routine, but with
    /// inverse values. Same thing about the MMU as the previous
    /// key presses routines
    pub fn key_up(&mut self, key: Key) {
        match key {
            Key::Right  => { self.keys[1] |= 0x1 },
            Key::Left   => { self.keys[1] |= 0x2 },
            Key::Up     => { self.keys[1] |= 0x4 },
            Key::Down   => { self.keys[1] |= 0x8 },
            Key::A      => { self.keys[0] |= 0x1 },
            Key::B      => { self.keys[0] |= 0x2 },
            Key::Select => { self.keys[0] |= 0x5 },
            Key::Start  => { self.keys[0] |= 0x8 },
        }
    }
}