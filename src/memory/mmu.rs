use std::path;
use memory::mbc;
use cpu::timer::Timer;
use frontend::keypad::Keypad;
use gpu::gpu::GPU;

const WRAM_SIZE: usize = 0x8000;
const ZRAM_SIZE: usize = 0x7F;

pub struct MMU {
    wram: [u8; WRAM_SIZE],
    zram: [u8; ZRAM_SIZE],

    pub inte: u8,
    pub intf: u8,

    pub timer: Timer,
    pub keypad: Keypad,

    pub gpu: GPU,

    wrambank: usize,

    pub mbc: Box<mbc::MBC+'static>
}

impl MMU {
    pub fn new(rom_file: &str) -> MMU {
        let mbc = mbc::load_mbc(path::PathBuf::from(rom_file)).unwrap();

        let mut mmu = MMU {
            wram: [0; WRAM_SIZE],
            zram: [0; ZRAM_SIZE],

            wrambank: 1,

            inte: 0,
            intf: 0,

            timer: Timer::new(),
            keypad: Keypad::new(),
            gpu: GPU::new(),
            mbc: mbc
        };

        mmu.reset();

        mmu
    }

    fn reset(&mut self) {
        self.wb(0xFF05, 0);
        self.wb(0xFF06, 0);
        self.wb(0xFF07, 0);
        self.wb(0xFF10, 0x80);
        self.wb(0xFF11, 0xBF);
        self.wb(0xFF12, 0xF3);
        self.wb(0xFF14, 0xBF);
        self.wb(0xFF16, 0x3F);
        self.wb(0xFF16, 0x3F);
        self.wb(0xFF17, 0);
        self.wb(0xFF19, 0xBF);
        self.wb(0xFF1A, 0x7F);
        self.wb(0xFF1B, 0xFF);
        self.wb(0xFF1C, 0x9F);
        self.wb(0xFF1E, 0xFF);
        self.wb(0xFF20, 0xFF);
        self.wb(0xFF21, 0);
        self.wb(0xFF22, 0);
        self.wb(0xFF23, 0xBF);
        self.wb(0xFF24, 0x77);
        self.wb(0xFF25, 0xF3);
        self.wb(0xFF26, 0xF1);
        self.wb(0xFF40, 0x91);
        self.wb(0xFF42, 0);
        self.wb(0xFF43, 0);
        self.wb(0xFF45, 0);
        self.wb(0xFF47, 0xFC);
        self.wb(0xFF48, 0xFF);
        self.wb(0xFF49, 0xFF);
        self.wb(0xFF4A, 0);
        self.wb(0xFF4B, 0);
    }

    pub fn do_cycle(&mut self, ticks: u32) {
        self.timer.do_cycle(ticks);
        self.intf |= self.timer.interrupt;
        self.timer.interrupt = 0;

        self.intf |= self.keypad.interrupt;
        self.keypad.interrupt = 0;

        self.gpu.do_cycle(ticks);
        self.intf |= self.gpu.interrupt;
        self.gpu.interrupt = 0;
    }

    pub fn rb(&mut self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x7FFF => {
                self.mbc.read_rom(address)
            },

            0x8000 ... 0x9FFF => {
                self.gpu.rb(address)
            },

            0xA000 ... 0xBFFF => {
                self.mbc.read_ram(address)
            },

            0xC000 ... 0xCFFF | 0xE000 ... 0xEFFF => {
                self.wram[address as usize & 0x0FFF]
            },

            0xD000 ... 0xDFFF | 0xF000 ... 0xFDFF => {
                self.wram[(self.wrambank * 0x1000) | address as usize & 0x0FFF]
            },

            0xFE00 ... 0xFE9F => {
                self.gpu.rb(address)
            },

            0xFF00 => {
                self.keypad.rb()
            },

            0xFF01 ... 0xFF02 => {
                // Serial unimplemented
                0x0
            },

            0xFF04 ... 0xFF07 => {
                self.timer.rb(address)
            },

            0xFF0F => {
                self.intf
            },

            0xFF10 ... 0xFF3F => {
                // Sound unimplemented
                0x0
            },

            0xFF4D => {
                0
            },

            0xFF40 ... 0xFF4F => {
                self.gpu.rb(address)
            },

            0xFF68 ... 0xFF6B => {
                self.gpu.rb(address)
            },

            0xFF70 => {
                self.wrambank as u8
            },

            0xFF80 ... 0xFFFE => {
                self.zram[address as usize & 0x007F]
            },

            0xFFFF => {
                self.inte
            },

            _ => 0,
        }
    }

    pub fn rw(&mut self, address: u16) -> u16 {
        (self.rb(address) as u16) |
            ((self.rb(address + 1) as u16) << 8)
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x7FFF =>  {
                self.mbc.write_rom(address, value)
            },

            0x8000 ... 0x9FFF => {
                self.gpu.wb(address, value)
            },

            0xA000 ... 0xBFFF => {
                self.mbc.write_ram(address, value)
            },

            0xC000 ... 0xCFFF | 0xE000 ... 0xEFFF => {
                self.wram[address as usize & 0x0FFF] = value
            },

            0xD000 ... 0xDFFF | 0xF000 ... 0xFDFF => {
                self.wram[(self.wrambank * 0x1000) | (address as usize & 0x0FFF)] = value
            },

            0xFE00 ... 0xFE9F => {
                self.gpu.wb(address, value)
            },

            0xFF00 => {
                self.keypad.wb(value)
            },

            0xFF04 ... 0xFF07 => {
                self.timer.wb(address, value)
            },

            0xFF10 ... 0xFF3F => {
                // panic!("Sound unimplemented")
            },

            0xFF46 => {
                self.oamdma(value)
            },

            0xFF4D => {
                if value & 0x1 == 0x1 {
                    //self.speed_switch_req = true;
                }
            },

            0xFF40 ... 0xFF4F => {
                self.gpu.wb(address, value)
            },

            0xFF68 ... 0xFF6B => {
                self.gpu.wb(address, value)
            },

            0xFF0F => {
                self.intf = value
            },

            0xFF70 => {
                self.wrambank = match value & 0x7 {
                    0 => 1,
                    n => n as usize
                };
            },

            0xFF80 ... 0xFFFE => {
                self.zram[address as usize & 0x007F] = value
            },

            0xFFFF => {
                self.inte = value
            },

            0xFF01 ... 0xFF02 => {
                // panic!("Serial port unimplemented")
            },

            _ => panic!("Unimplemented memory instruction"),
        };
    }

    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8);
        self.wb(address + 1, (value >> 8) as u8);
    }

    fn oamdma(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for i in 0 .. 0xA0 {
            let b = self.rb(base + i);
            self.wb(0xFE00 + i, b);
        }
    }
}
