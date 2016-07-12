use cpu::registers::RegisterSet;
use cpu::clock::Clock;
use memory::mmu::MMU;

pub struct Z80 {
    clock: Clock,
    mmu: MMU,
    registers: RegisterSet
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80 {
            clock: Clock::new(),
            registers: RegisterSet::new(),
            mmu: MMU::new()
        }
    }

    pub fn step(&self) {

    }

    pub fn interrupt(&self) {

    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.mmu.load_rom(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::mmu::MMU;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn it_instantiates() {
        let mut cpu = Z80::new();
    }
}