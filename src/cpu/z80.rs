use cpu::registers::RegisterSet;
use cpu::clock::Clock;
use memory::mmu::MMU;
use std::cell::RefCell;
use std::rc::Rc;
use std::boxed::Box;

pub struct Z80 {
    clock: Clock,
    mmu: Option<Box<Rc<RefCell<MMU>>>>,
    registers: Option<Rc<RefCell<RegisterSet>>>
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80 {
            clock: Clock::new(),
            registers: None,
            mmu: None
        }
    }

    pub fn set_registers(&mut self, registers: Rc<RefCell<RegisterSet>>) {
        self.registers = Some(registers);
    }

    pub fn set_mmu(&mut self, mmu: Box<Rc<RefCell<MMU>>>) {
        self.mmu = Some(mmu);
    }

    pub fn step(&self) {

    }

    pub fn interrupt(&self) {

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
        let mmu = Box::new(Rc::new(RefCell::new(MMU::new())));

        let mut cpu = Z80::new();

        cpu.set_mmu(mmu);
    }
}