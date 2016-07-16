use memory::mbc::{MBC, ram_size};

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_on: bool,
    ram_mode: bool,
    rombank: usize,
    rambank: usize,
}

impl MBC1 {
    pub fn new(data: Vec<u8>,) -> MBC1 {
        let ramsize = match data[0x147] {
            0x02 => ram_size(data[0x149]),
            0x03 => ram_size(data[0x149]),
            _ => 0,
        };

        MBC1 {
            rom: data,
            ram: ::std::iter::repeat(0u8).take(ramsize).collect(),
            ram_on: false,
            ram_mode: false,
            rombank: 1,
            rambank: 0,
        }
    }
}

impl MBC for MBC1 {
    fn read_rom(&self, a: u16) -> u8 {

        let idx = if a < 0x4000 { a as usize }
            else { self.rombank * 0x4000 | ((a as usize) & 0x3FFF) };
        *self.rom.get(idx).unwrap_or(&0)
    }

    fn read_ram(&self, a: u16) -> u8 {
        if !self.ram_on { return 0 }
        let rambank = if self.ram_mode { self.rambank } else { 0 };
        self.ram[(rambank * 0x2000) | ((a & 0x1FFF) as usize)]
    }

    fn write_rom(&mut self, a: u16, v: u8) {
        match a {
            0x0000 ... 0x1FFF => { self.ram_on = v == 0x0A; },
            0x2000 ... 0x3FFF => {
                self.rombank = (self.rombank & 0x60) | match (v as usize) & 0x1F { 0 => 1, n => n }
            },
            0x4000 ... 0x5FFF => {
                if !self.ram_mode {
                    self.rombank = self.rombank & 0x1F | (((v as usize) & 0x03) << 5)
                } else {
                    self.rambank = (v as usize) & 0x03;
                }
            },
            0x6000 ... 0x7FFF => { self.ram_mode = (v & 0x01) == 0x01; },
            _ => panic!("Could not write to {:04X} (MBC1)", a),
        }
    }

    fn write_ram(&mut self, a: u16, v: u8) {
        if !self.ram_on { return }
        let rambank = if self.ram_mode { self.rambank } else { 0 };
        self.ram[(rambank * 0x2000) | ((a & 0x1FFF) as usize)] = v;
    }
}
