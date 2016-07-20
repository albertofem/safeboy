use memory::mbc::{MBC, ram_size};

/// MBC 1
///
/// This is the first and most primitive MBC1 used in games
/// It holds a maximum of 2MB of ROM and 32kb of RAM
pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_on: bool,
    ram_mode: bool,
    rom_bank: usize,
    ram_bank: usize,
}


impl MBC1 {
    pub fn new(data: Vec<u8>,) -> MBC1 {
        let ramsize = match data[0x147] {
            0x02 => ram_size(data[0x149]),
            0x03 => ram_size(data[0x149]),
            _ => 0,
        };

        let mut initial_ram = Vec::with_capacity(ramsize);

        for _i in 0..ramsize {
            initial_ram.push(0u8);
        }

        MBC1 {
            rom: data,
            ram: initial_ram,
            ram_on: false,
            ram_mode: false,
            rom_bank: 1,
            ram_bank: 0,
        }
    }
}

impl MBC for MBC1 {
    fn read_rom(&self, address: u16) -> u8 {

        let index =
            if address < 0x4000 {
                address as usize
            } else {
                self.rom_bank * 0x4000 | ((address as usize) & 0x3FFF)
            };

        let not_found_value = 0u8;

        // get this position value, or 0 if it's not found
        let rom_byte = self.rom.get(index).unwrap_or(&not_found_value);

        *rom_byte
    }

    fn write_rom(&mut self, a: u16, v: u8) {
        match a {
            0x0000 ... 0x1FFF => {
                self.ram_on = v == 0x0A;
            },

            0x2000 ... 0x3FFF => {
                self.rom_bank =
                    (self.rom_bank & 0x60) |
                    match (v as usize) & 0x1F {
                        0 => 1,
                        n => n
                    }
            },

            0x4000 ... 0x5FFF => {
                if !self.ram_mode {
                    self.rom_bank = self.rom_bank & 0x1F | (((v as usize) & 0x03) << 5)
                } else {
                    self.ram_bank = (v as usize) & 0x03;
                }
            },

            0x6000 ... 0x7FFF => {
                self.ram_mode = (v & 0x01) == 0x01;
            },

            _ => panic!("Could not write to {:04X} (MBC1)", a),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_on {
            return 0
        }

        let ram_bank = if self.ram_mode {
            self.ram_bank
        } else {
            0
        };

        self.ram[(ram_bank * 0x2000) | ((address & 0x1FFF) as usize)]
    }

    fn write_ram(&mut self, a: u16, v: u8) {
        if !self.ram_on {
            return
        }

        let ram_bank = if self.ram_mode {
            self.ram_bank
        } else {
            0
        };

        self.ram[(ram_bank * 0x2000) | ((address & 0x1FFF) as usize)] = v;
    }
}
