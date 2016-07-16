use std::io::prelude::*;
use std::fs::File;

mod mbc0;
mod mbc1;

/// Memory Banking Controller
///
/// This is the MBC interface, which is used by the
/// MMU to read the ROM data (cartdriged). It's implementation
/// is based on the actual MBC found in each game
///
/// Games could ship their custom MBC hardwired into the
/// cartdriged, in order to workaround the physical limitations
/// of the GameBoy hardware
///
/// Each MBC implemented an algorithm to incrase the available
/// memory of the GameBoy without needed to upgrade the hardware.
///
/// There is about 30 MBC types out there, but we only implemented
/// the first two ones: MBC0 (no-MBC) and MBC1
pub trait MBC {
    /// Reads ROM from the give address
    fn read_rom(&self, address: u16) -> u8;

    /// Writes data to the ROM
    fn write_rom(&mut self, address: u16, value: u8);

    fn read_ram(&self, address: u16) -> u8;
    fn write_ram(&mut self, address: u16, value: u8);
}

/// Loads a new MBC
///
/// This method will detect which kind of MBC the game has
/// and instantiate the proper implementation.
///
/// The result is returned as a Box, which contains the raw
/// game data allocaated in the stack
pub fn load_mbc(rom_file: &str) -> Result<Box<MBC+'static>, String> {
    let mut data = vec![];
    let mut file = File::open(rom_file).unwrap();

    // read all bytes through the end, data now contains all
    // raw rom bytes
    let size = file.read_to_end(&mut data).unwrap();

    // in order to know what kind of MBC we are working with,
    // we need to read this address space in the ROM, which
    // contains the cartdridge type
    let mbc_type = data[0x147];

    match mbc_type {
        0x00 => {
            let mbc = mbc0::MBC0::new(data);
            Ok(Box::new(mbc) as Box<MBC>)
        },

        0x01 ... 0x03 =>  {
            let mbc = mbc1::MBC1::new(data);
            Ok(Box::new(mbc) as Box<MBC>)
        },

        _ => Err(format!("Unsupported MBC: {0:x}", mbc_type)),
    }
}

fn ram_size(v: u8) -> usize {
    match v {
        1 => 0x800,
        2 => 0x2000,
        3 => 0x8000,
        4 => 0x20000,
        _ => 0,
    }
}