use std::fs::File;
use std::io::Read;

pub struct Cartridge {
    data: Vec<u8>
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            data: vec!()
        }
    }

    pub fn read(&mut self, rom_file: &str) -> () {
        let mut file = File::open(rom_file).unwrap();
        let mut data = vec!();

        file.read_to_end(&mut data);

        self.data = data;
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_instantiates() {
        let _ = Cartridge::new();
    }

    #[test]
    fn it_open_rom_file() {
        let mut cartridge = Cartridge::new();

        cartridge.read("./data/dmg.rom");

        let data = cartridge.data();

        for i in 0..data.len() {
            assert_eq!(data[i], data[i]);
        }
    }
}
