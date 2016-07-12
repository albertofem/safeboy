use cartridge::cartridge::Cartridge;
use cpu::z80::Z80;
use cpu::registers::RegisterSet;
use memory::mmu::MMU;
use gpu::gpu::GPU;
use display::display::{Display, Event};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Gameboy<'a> {
    cpu: Z80<'a>,
    cartridge: Cartridge,
    gpu: GPU,
    display: Display
}

impl<'a> Gameboy<'a> {
    pub fn new() -> Gameboy<'a> {
        Gameboy {
            cpu: Z80::new(),
            cartridge: Cartridge::new(),
            gpu: GPU::new(),
            display: Display::new(),
        }
    }

    pub fn run_game(mut self, rom_file: &str) -> () {
        self.cartridge.read(rom_file);
        self.cpu.load_rom(self.cartridge.data());
        self.display.initialize(rom_file);

        loop {
            match self.display.poll_events() {
                Event::Closed => {
                    println!("Closing Gameboy, safe travels!");
                    break;
                },
                _ => ()
            }

            self.cpu.step();
            self.gpu.step();
            self.cpu.interrupt();

            self.display.draw();
        }
    }
}