use std::io::prelude::*;
use std::fs::File;
use std::path;

mod mbc0;

const TITLE_START : u16 = 0x134;
const TITLE_SIZE : u16 = 11;

pub trait MBC : Send {
    fn read_rom(&self, a: u16) -> u8;
    fn write_rom(&mut self, a: u16, v: u8);

    fn read_ram(&self, a: u16) -> u8;
    fn write_ram(&mut self, a: u16, v: u8);

    fn rom_name(&self) -> String {
        let mut result = String::with_capacity(TITLE_SIZE as usize);

        for i in 0..TITLE_SIZE {
            match self.read_rom(TITLE_START + i) {
                0 => break,
                v => result.push(v as char),
            }
        }

        result
    }
}

pub fn load_mbc(rom_file: path::PathBuf) -> Result<Box<MBC+'static>, String> {
    let mut data = vec![];
    let mut file = File::open(rom_file).unwrap();

    file.read_to_end(&mut data);

    if data.len() < 0x150 {
        return Err("Rom size too small!".to_string())
    }

    try!(check_checksum(&data));

    let mbc_type = data[0x147];

    match mbc_type {
        0x00 => {
            let mbc = mbc0::MBC0::new(data);
            Ok(Box::new(mbc) as Box<MBC>)
        },
        _ => Err(format!("Unsupported MBC: {0:x}", mbc_type)),
    }
}

fn check_checksum(data: &[u8]) -> Result<(), &'static str> {
    let mut value: u8 = 0;

    for i in 0x134 .. 0x14D {
        value = value.wrapping_sub(data[i]).wrapping_sub(1);
    }

    if data[0x14D] != value {
        return Err("MBC checksum is broken")
    }

    Ok(())
}