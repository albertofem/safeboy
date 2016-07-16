pub struct Keypad {
    keys: [u8; 2],
    column: u8,
    pub interrupt: u8
}

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

    pub fn rb(&self) -> u8 {
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

    pub fn wb(&mut self, value: u8) {
        self.column = value & 0x30;
    }

    pub fn key_down(&mut self, key: Key) {
        match key {
            Key::A => {
                self.keys[0] &= 0xE
            },
            Key::B => {
                self.keys[0] &= 0xD
            },
            Key::Select => {
                self.keys[0] &= 0xB
            },
            Key::Start => {
                self.keys[0] &= 0x7
            },
            Key::Up => {
                self.keys[1] &= 0xB
            },
            Key::Down => {
                self.keys[1] &= 0x7
            },
            Key::Right => {
                self.keys[1] &= 0xE
            },
            Key::Left => {
                self.keys[1] &= 0xD
            }
        }

        self.interrupt |= 0x10;
    }

    pub fn key_up(&mut self, key: Key) {
        match key {
            Key::A => {
                self.keys[0] |= 0x1
            },
            Key::B => {
                self.keys[0] |= 0x2
            },
            Key::Select => {
                self.keys[0] |= 0x8
            },
            Key::Start => {
                self.keys[0] |= 0x4
            },
            Key::Up => {
                self.keys[1] |= 0x8
            },
            Key::Down => {
                self.keys[1] |= 0x4
            },
            Key::Right => {
                self.keys[1] |= 0x1
            },
            Key::Left => {
                self.keys[1] |= 0x2
            }
        }
    }
}