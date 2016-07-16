pub struct Keypad {
    keys: [u8; 2],
    column: u8,
    pub interrupt: u8
}

#[derive(Debug, Clone, Copy)]
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
        let mut keypad = Keypad {
            keys: [
                0x0F,
                0x0F
            ],
            column: 0,
            interrupt: 0
        };

        keypad.update();

        keypad
    }

    fn update(&mut self) {
        self.column &= 0x30;

        if self.column & 0x10 == 0x10 {
            self.column |= self.keys[0];
        }

        if self.column & 0x20 == 0x20 {
            self.column |= self.keys[1];
        }
    }

    pub fn rb(&self) -> u8 {
        self.column
    }

    pub fn wb(&mut self, value: u8) {
        self.column = value;
        self.update();
    }

    pub fn key_down(&mut self, key: Key) {
        match key {
            Key::A => {
                self.keys[0] &= !(1 << 0)
            },
            Key::B => {
                self.keys[0] &= !(1 << 1)
            },
            Key::Select => {
                self.keys[0] &= !(1 << 2)
            },
            Key::Start => {
                self.keys[0] &= !(1 << 3)
            },
            Key::Up => {
                self.keys[1] &= !(1 << 2)
            },
            Key::Down => {
                self.keys[1] &= !(1 << 3)
            },
            Key::Right => {
                self.keys[1] &= !(1 << 0)
            },
            Key::Left => {
                self.keys[1] &= !(1 << 1)
            }
        }

        self.interrupt |= 0x10;
        self.update();
    }

    pub fn key_up(&mut self, key: Key) {
        match key {
            Key::A => {
                self.keys[0] |= 1 << 0
            },
            Key::B => {
                self.keys[0] |= 1 << 1
            },
            Key::Select => {
                self.keys[0] |= 1 << 2
            },
            Key::Start => {
                self.keys[0] |= 1 << 3
            },
            Key::Up => {
                self.keys[1] |= 1 << 2
            },
            Key::Down => {
                self.keys[1] |= 1 << 3
            },
            Key::Right => {
                self.keys[1] |= 1 << 0
            },
            Key::Left => {
                self.keys[1] |= 1 << 1
            }
        }
        self.update();
    }
}