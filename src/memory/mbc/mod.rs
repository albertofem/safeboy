use std::io::prelude::*;
use std::fs::File;
use std::path;

mod mbc0;

pub trait MBC : Send {
    fn readrom(&self, a: u16) -> u8;
    fn readram(&self, a: u16) -> u8;
    fn writerom(&mut self, a: u16, v: u8);
    fn writeram(&mut self, a: u16, v: u8);

    fn romname(&self) -> String {
        const TITLE_START : u16 = 0x134;
        const CGB_FLAG : u16 = 0x143;

        let title_size = match self.readrom(CGB_FLAG) & 0x80 {
            0x80 => 11,
            _ => 16,
        };

        let mut result = String::with_capacity(title_size as usize);

        for i in 0..title_size {
            match self.readrom(TITLE_START + i) {
                0 => break,
                v => result.push(v as char),
            }
        }

        result
    }
}

pub fn get_mbc(file: path::PathBuf) -> ::StrResult<Box<MBC+'static>> {
    let mut data = vec![];

    try!(File::open(&file).and_then(|mut f| f.read_to_end(&mut data)).map_err(|_| "Could not read ROM"));

    if data.len() < 0x150 { return Err("Rom size to small"); }

    try!(check_checksum(&data));

    match data[0x147] {
        0x00 => mbc0::MBC0::new(data).map(|v| Box::new(v) as Box<MBC>),
        _ => { Err("Unsupported MBC type") },
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

fn check_checksum(data: &[u8]) -> ::StrResult<()> {
    let mut value: u8 = 0;
    for i in 0x134 .. 0x14D {
        value = value.wrapping_sub(data[i]).wrapping_sub(1);
    }
    match data[0x14D] == value
        {
            true => Ok(()),
            false => Err("Cartridge checksum is invalid"),
        }
}

#[cfg(test)]
mod test {
    #[test]
    fn checksum_zero() {
        let mut data = vec![0; 0x150];
        data[0x14D] = -(0x14D_i32 - 0x134_i32) as u8;
        super::check_checksum(&data).unwrap();
    }

    #[test]
    fn checksum_ones() {
        let mut data = vec![1; 0x150];
        data[0x14D] = (-(0x14D_i32 - 0x134_i32) * 2) as u8;
        super::check_checksum(&data).unwrap();
    }
}
