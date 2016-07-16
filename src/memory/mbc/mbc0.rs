use memory::mbc::MBC;

pub struct MBC0 {
    rom: Vec<u8>,
}

/// This is the implementation for no MBC
///
/// Some games didn't need memory banking, so this
/// MBC implementation actually maps the requested
/// ROM addresses 1:1 with the ROM data
impl MBC0 {
    pub fn new(data: Vec<u8>) -> MBC0 {
        MBC0 {
            rom: data
        }
    }
}

impl MBC for MBC0 {
    fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write_rom(&mut self, _address: u16, _value: u8) {
        // we don't need to write anything since
        // there is no physical ROM in this MBC
        ()
    }

    fn read_ram(&self, _address: u16) -> u8 {
        0
    }

    fn write_ram(&mut self, _address: u16, _value: u8) {
        ()
    }
}
