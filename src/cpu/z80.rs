use cpu::registers::RegisterSet;
use cpu::clock::Clock;
use memory::mmu::MMU;

pub struct Z80<'a> {
    clock: Clock,
    registers: RegisterSet,
    mmu: Option<&'a MMU>
}

impl<'a> Z80<'a> {
    pub fn new() -> Z80<'a> {
        Z80 {
            clock: Clock::new(),
            registers: RegisterSet::new(),
            mmu: None
        }
    }

    pub fn set_mmu(&mut self, mmu: &'a MMU) {
        self.mmu = Some(mmu);
    }

    pub fn step(&mut self) {

    }

    pub fn interrupt(&mut self) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::mmu::MMU;

    #[test]
    fn it_instantiates() {
        let mmu = MMU::new();

        let mut cpu = Z80::new();

        cpu.set_mmu(&mmu);
    }
}