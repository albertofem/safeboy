use memory::mbc::MBC;

pub struct MBC0 {
    rom: Vec<u8>,
}

impl MBC0 {
    pub fn new(data: Vec<u8>) -> MBC0 {
        MBC0 {
            rom: data
        }
    }
}

impl MBC for MBC0 {
    fn read_rom(&self, a: u16) -> u8 {
        self.rom[a as usize]
    }

    fn write_rom(&mut self, _a: u16, _v: u8) {
        ()
    }

    fn read_ram(&self, _a: u16) -> u8 {
        0
    }

    fn write_ram(&mut self, _a: u16, _v: u8) {
        ()
    }
}
