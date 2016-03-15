use cartridge::cartridge::Cartridge;
use cpu::z80::Z80;
use memory::mmu::MMU;
use display::display::{Display, Event};

pub struct Gameboy {
    cpu: Z80,
    cartridge: Cartridge,
    mmu: MMU,
    display: Display
}

impl Gameboy {
    pub fn new() -> Gameboy {
        Gameboy {
            cpu: Z80::new(),
            cartridge: Cartridge::new(),
            mmu: MMU::new(),
            display: Display::new()
        }
    }

    pub fn run_game(&mut self, rom_file: &str) -> () {
        self.cartridge.read(rom_file);
        self.mmu.load_rom(self.cartridge.data());
        self.display.initialize(rom_file);

        loop {
            self.display.draw();

            match self.display.poll_events() {
                Event::Closed => {
                    println!("Closing Gameboy, safe travels!");
                    break;
                },
                _ => ()
            }
        }
    }
}