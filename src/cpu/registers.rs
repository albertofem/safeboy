use cpu::clock::Clock;

pub struct RegisterSet {
    // 8-bit registers
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    // flag register
    f: u8,

    // 16-bit registers
    pc: u16,
    sp: u16,

    clock: Clock
}

impl RegisterSet {
    pub fn new() -> RegisterSet {
        RegisterSet {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            
            f: 0,
            
            pc: 0,
            sp: 0,
            clock: Clock::new()
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        
        self.f = 0;
        
        self.pc = 0;
        self.sp = 0;
    }

    fn nop(&mut self) {
        self.clock.m = 1;
        self.clock.t = 4;
    }

    pub fn exec(&mut self, opcode: u8) {
        match opcode {
            // 00
            0x00 => self.nop(),              // NOP
            0x01 => unimplemented!(),
            0x02 => unimplemented!(),
            0x03 => unimplemented!(),
            0x04 => unimplemented!(),
            0x05 => unimplemented!(),
            0x06 => unimplemented!(),
            0x07 => unimplemented!(),
            0x08 => unimplemented!(),
            0x09 => unimplemented!(),
            0x0a => unimplemented!(),
            0x0b => unimplemented!(),
            0x0c => unimplemented!(),
            0x0d => unimplemented!(),
            0x0e => unimplemented!(),
            0x0f => unimplemented!(),

            // out of range
            _    => panic!("opcode out of range")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_instantiates() {
        let _ = RegisterSet::new();
    }

    #[test]
    fn it_executes_nop() {
        let mut registers = RegisterSet::new();

        registers.exec(0x00);

        assert_eq!(1, registers.clock.m);
        assert_eq!(4, registers.clock.t);
    }

    #[test]
    fn it_resets() {
        let mut registers = RegisterSet::new();

        registers.reset();

        assert_eq!(0, registers.a);
    }
}