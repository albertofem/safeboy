use cpu::registers::RegisterSet;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MMU {
    bios: [u8; 256],

    rom: Vec<u8>,
    cart_type: u8,

    mbc: MBC,

    rom_offs: u16,
    ram_offs: u16,

    eram: [u16; 32768],
    wram: [u16; 8192],
    zram: [u16; 127],

    in_bios: bool,

    ie: u16,
    interrupt_flag: u16,

    registers: Option<Rc<RefCell<RegisterSet>>>
}

pub struct MBC {
    rom_bank: u16,
    ram_bank: u16,
    ram_on: u8,
    mode: u16
}

impl MBC {
    pub fn new() -> MBC {
        MBC {
            rom_bank: 0,
            ram_bank: 0,
            ram_on: 0,
            mode: 0
        }
    }
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            // GameBoy bios is hard-wired into the memory
            bios: [
                0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26, 0xFF, 0x0E,
                0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0,
                0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
                0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06, 0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9,
                0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21, 0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20,
                0xF9, 0x2E, 0x0F, 0x18, 0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04,
                0x1E, 0x02, 0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
                0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64, 0x20, 0x06,
                0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xF2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15, 0x20, 0xD2, 0x05, 0x20,
                0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB, 0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17,
                0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9, 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
                0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
                0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
                0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3c, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x4C,
                0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x20, 0xFE, 0x23, 0x7D, 0xFE, 0x34, 0x20,
                0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x20, 0xFE, 0x3E, 0x01, 0xE0, 0x50
            ],

            rom: vec!(),
            cart_type: 0,

            mbc: MBC::new(),

            rom_offs: 0x4000,
            ram_offs: 0,

            eram: [0; 32768],
            wram: [0; 8192],
            zram: [0; 127],

            // defaults to bios
            in_bios: true,  

            ie: 0,
            interrupt_flag: 0,

            registers: None
        }
    }

    pub fn load_rom(&mut self, data: Vec<u8>) -> () {
        self.rom = data;
        self.cart_type = self.rom[0x0147];

        println!("MMU loaded");
        println!("Cart type is: {}", self.cart_type);
    }

    pub fn set_registers(&mut self, registers: Rc<RefCell<RegisterSet>>) {
        self.registers = Some(registers);
    }

    pub fn write_byte(&mut self, address: u16, value: u16) {
        match address & 0xF000 {
            0x0000 | 0x1000 => {
                if self.cart_type == 1 {
                    if value & 0xF == 0xA {
                        self.mbc.ram_on = 1;
                    } else {
                        self.mbc.ram_on = 0;
                    }
                }
            },
            0x2000 | 0x3000 => {
                if self.cart_type == 1 {
                    self.mbc.rom_bank &= 0x60;

                    let mut calc_value = value;

                    calc_value &= 0x1F;

                    if calc_value == 0 {
                        calc_value = 1
                    }

                    self.mbc.rom_bank |= calc_value;
                    self.rom_offs = self.mbc.rom_bank * 0x4000;
                }
            },
            0x4000 | 0x5000 => {
                if self.cart_type == 1 {
                    if self.mbc.mode == 1 {
                        self.mbc.ram_bank = value & 3;
                        self.mbc.rom_bank |= (value & 3) << 5;
                        self.rom_offs = self.mbc.rom_bank * 0x4000;
                    }
                }
            },
            0x6000 | 0x7000 => {
                if self.cart_type == 1 {
                    self.mbc.mode = value & 1;
                }
            },
            0x8000 | 0x9000 => {
                // TODO: GPU
            },
            0xA000 | 0xB000 => {
                let search_address = self.ram_offs + (address & 0x1FFF);

                self.eram[search_address as usize] = value;
            },
            0xC000 | 0xD000 => {
                let search_address = address & 0x1FFF;
                self.wram[search_address as usize] = value;
            },
            0xF000 => {
                match address & 0xF000 {
                    0x000 | 0x100 | 0x200 | 0x300 | 0x400 | 0x500 |
                    0x600 | 0x700 | 0x800 | 0x900 | 0xA00 | 0xB00 => {
                        let search_address = address & 0x1FFF;
                        self.wram[search_address as usize] = value;
                    },
                    0xE00 => {
                        if (address & 0xFF) < 0xA0 {
                            // GPU.oam
                        }
                        // update gpu oam
                    },
                    0xF00 => {
                        if address == 0xFFFF {
                            self.ie = value;
                        } else if address > 0xFF7F {
                            let search_address = address & 0x7F;
                            self.zram[search_address as usize] = value;
                        } else {
                            match address & 0xF0 {
                                0x00 => {
                                    match address & 0xF {
                                        0 => {
                                            // TODO: Keys
                                        },
                                        4 | 5 | 6 | 7 => {
                                            // TODO: Timer
                                        },
                                        15 => {
                                            self.interrupt_flag = value;
                                        },
                                        _ => panic!("Invalid memory address")
                                    }
                                },
                                _ => panic!("Invalid memory address")
                            }
                        }
                    },
                    _ => panic!("Invalid memory address")
                }
            }
            _ => panic!("Invalid MMU write!")
        }
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        if self.registers.is_none() {
            panic!("MMU registers are not properly initialized");
        }

        match address & 0xF000 {
            // rom bank 0
            0x0000 => self.read_0x0000(address),
            _      => unreachable!()
        }
    }

    fn read_0x0000(&mut self, address: u16) -> u8 {
        if self.in_bios == true {
            let registers = self.registers.as_mut().unwrap();

            if address < 0x0100 {
                return self.bios[address as usize];
            } else if registers.borrow_mut().pc == 0x100 {
                self.in_bios = false;
                return 0x0;
            }
        } else {
            return self.rom[address as usize];
        }

        return 0x0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpu::registers::RegisterSet;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn it_instantiates_mmu() {
        let _ = MMU::new();
    }

    #[test]
    fn it_instantiates_mbc() {
        let _ = MBC::new();
    }

    #[test]
    fn it_reads_byte_from_bios_address() {
        let registers = RegisterSet::new();
        let mut mmu = MMU::new();

        mmu.set_registers(Rc::new(RefCell::new(registers)));

        // test first 5 entries
        assert_eq!(mmu.read_byte(0x0000), 0x31);
        assert_eq!(mmu.read_byte(0x0001), 0xFE);
        assert_eq!(mmu.read_byte(0x0002), 0xFF);
        assert_eq!(mmu.read_byte(0x0003), 0xAF);
        assert_eq!(mmu.read_byte(0x0004), 0x21);
    }

    #[test]
    fn it_reads_byte_from_rom() {
        let mut registers = RegisterSet::new();
        let mut mmu = MMU::new();

        mmu.set_registers(Rc::new(RefCell::new(registers)));

        // get outside the bios setting PC
        registers.pc = 0x0100;

        // test first 5 entries
        assert_eq!(mmu.read_byte(0x0000), 0x31);
    }
}