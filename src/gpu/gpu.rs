pub struct GPU {
    pub interrupt: u8
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            interrupt: 0
        }
    }

    pub fn do_cycle(&mut self, ticks: u32) {

    }

    pub fn rb(&self, address: u16) -> u8 {
        0x0
    }

    pub fn wb(&mut self, address: u16, value: u8) {

    }
}