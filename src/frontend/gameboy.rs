use cartridge::cartridge::Cartridge;
use cpu::z80::Z80;
use cpu::registers::RegisterSet;
use memory::mmu::MMU;
use gpu::gpu::GPU;
use display::display::{Display, Event};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Gameboy {
    cpu: Z80,
    display: Display
}

impl Gameboy {
    pub fn new(rom_file: &str) -> Gameboy {
        Gameboy {
            cpu: Z80::new(rom_file),
            display: Display::new(),
        }
    }

    pub fn run(&mut self) -> () {
        self.display.initialize();

        loop {
            match self.display.poll_events() {
                Event::Closed => {
                    println!("Closing Gameboy, safe travels!");
                    break;
                },
                _ => ()
            }

            self.display.draw([]);
        }
    }
}