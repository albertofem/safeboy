use cartridge::cartridge::Cartridge;
use cpu::z80::Z80;
use memory::mmu::MMU;
use gpu::gpu::GPU;
use display::display::{Display, Event};

pub struct Gameboy<'a> {
    cpu: Z80<'a>,
    cartridge: Cartridge,
    mmu: MMU,
    gpu: GPU,
    display: Display
}

impl<'a> Gameboy<'a> {
    pub fn new() -> Gameboy<'a> {
        Gameboy {
            cpu: Z80::new(),
            cartridge: Cartridge::new(),
            mmu: MMU::new(),
            gpu: GPU::new(),
            display: Display::new()
        }
    }

    pub fn run_game(&'a mut self, rom_file: &str) -> () {
        self.cartridge.read(rom_file);
        self.mmu.load_rom(self.cartridge.data());
        self.display.initialize(rom_file);

        // initialize
        self.cpu.set_mmu(&self.mmu);

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