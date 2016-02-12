use cpu::registers::RegisterSet;
use cpu::clock::Clock;

pub struct Z80 {
    clock: Clock,
    registers: RegisterSet
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80 {
            clock: Clock::new(),
            registers: RegisterSet::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_instantiates() {
        let _ = Z80::new();
    }
}