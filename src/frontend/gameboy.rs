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
    cartridge: Cartridge,
    mmu: MMU,
    gpu: GPU,
    display: Display,
    registers: RegisterSet
}

impl Gameboy {
    pub fn new() -> Gameboy {
        Gameboy {
            cpu: Z80::new(),
            cartridge: Cartridge::new(),
            mmu: MMU::new(),
            gpu: GPU::new(),
            display: Display::new(),
            registers: RegisterSet::new()
        }
    }

    pub fn run_game(mut self, rom_file: &str) -> () {
        self.cartridge.read(rom_file);
        self.mmu.load_rom(self.cartridge.data());
        self.display.initialize(rom_file);

        let registers = Rc::new(RefCell::new(self.registers));

        // initialize registers
        self.mmu.set_registers(registers.clone());
        self.cpu.set_registers(registers.clone());

        let mmu = Box::new(Rc::new(RefCell::new(self.mmu)));

        // initialize cpu
        self.cpu.set_mmu(mmu);

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