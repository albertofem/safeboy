use cpu::registers::RegisterSet;
use cpu::clock::Clock;
use memory::mmu::MMU;
use cpu::registers::CpuFlag::{C, N, H, Z};
use frontend::keypad::Key;

const CPU_SPEED: u32 = 4_194_304;

pub struct Z80 {
    clock: Clock,
    mmu: MMU,
    registers: RegisterSet,
    halted: bool,
    ime: bool,
    setdi: u32,
    setei: u32,
}

impl Z80 {
    pub fn new(rom_file: &str) -> Z80 {
        Z80 {
            clock: Clock::new(),
            registers: RegisterSet::new(),
            mmu: MMU::new(rom_file),
            halted: false,
            ime: true,
            setei: 0,
            setdi: 0
        }
    }

    pub fn step(&mut self) {
        let mut cpu_ticks = 0;

        let cpu_speed = (CPU_SPEED / 1000 * 16) as u32;

        while cpu_ticks < cpu_speed {
            self.docycle();
            self.mmu.do_cycle(self.clock.t * 4);

            cpu_ticks += self.clock.t;
        }

        cpu_ticks -= cpu_speed;
    }

    fn docycle(&mut self) {
        self.updateime();

        match self.interrupt() {
            0 => {},
            n => {
                self.clock(1, n)
            },
        };

        if !self.halted {
            let opcode = self.fetch_byte();
            self.call(opcode)
        }
    }

    fn updateime(&mut self) {
        self.setdi = match self.setdi {
            2 => 1,
            1 => {
                self.ime = false;
                0
            },
            _ => 0,
        };
        self.setei = match self.setei {
            2 => 1,
            1 => {
                self.ime = true;
                0
            },
            _ => 0,
        };
    }

    fn interrupt(&mut self) -> u32 {
        if self.ime == false && self.halted == false {
            return 0
        }

        let triggered = self.mmu.inte & self.mmu.intf;

        if triggered == 0 {
            return 0
        }

        self.halted = false;

        if self.ime == false {
            return 0
        }

        self.ime = false;

        let n = triggered.trailing_zeros();

        if n >= 5 {
            panic!("Invalid interrupt triggered");
        }

        self.mmu.intf &= !(1 << n);

        let pc = self.registers.pc;

        self.push_stack(pc);

        self.registers.pc = 0x0040 | ((n as u16) << 3);

        return 4
    }

    fn push_stack(&mut self, value: u16) {
        self.registers.sp -= 2;
        self.mmu.ww(self.registers.sp, value);
    }

    fn pop_stack(&mut self) -> u16 {
        let res = self.mmu.rw(self.registers.sp);
        self.registers.sp += 2;
        res
    }

    fn fetch_byte(&mut self) -> u8 {
        let b = self.mmu.rb(self.registers.pc);
        self.registers.pc += 1;

        b
    }

    fn fetch_word(&mut self) -> u16 {
        let w = self.mmu.rw(self.registers.pc);
        self.registers.pc += 2;
        w
    }

    fn clock(&mut self, m: u32, t: u32) {
        self.clock.m = m;
        self.clock.t = t;
    }

    fn call(&mut self, opcode: u8) {
        let oldregs = self.registers;

        match opcode {
            0x00 => {
                self.clock(1, 4);
            },

            0x01 => {
                let v = self.fetch_word();
                self.registers.set_bc(v);
                self.clock(1, 3);
            },

            0x02 => {
                self.mmu.wb(self.registers.bc(), self.registers.a);
                self.clock(1, 2);
            },

            0x03 => {
                let v = self.registers.bc().wrapping_add(1);
                self.registers.set_bc(v);
                self.clock(1, 2);
            },

            0x04 => {
                self.registers.b = self.alu_inc(oldregs.b);
                self.clock(1, 1);
            },

            0x05 => {
                self.registers.b = self.alu_dec(oldregs.b);
                self.clock(1, 1);
            },
            
            0x06 => {
                self.registers.b = self.fetch_byte();
                self.clock(1, 2);
            },
            
            0x07 => {
                self.registers.a = self.alu_rlc(oldregs.a);
                self.registers.flag(Z, false);
                self.clock(1, 1);
            },

            0x08 => {
                let a = self.fetch_word();
                self.mmu.ww(a, self.registers.sp);
                self.clock(1, 5);
            },

            0x09 => {
                let v = self.registers.bc();
                self.alu_add16(v);
                self.clock(1, 2);
            },

            0x0A => {
                self.registers.a = self.mmu.rb(self.registers.bc());
                self.clock(1, 2);
            },

            0x0B => {
                let v = self.registers.bc().wrapping_sub(1);
                self.registers.set_bc(v);
                self.clock(1, 2);
            },

            0x0C => {
                self.registers.c = self.alu_inc(oldregs.c);
                self.clock(1, 1);
            },

            0x0D => {
                self.registers.c = self.alu_dec(oldregs.c);
                self.clock(1, 1);
            },

            0x0E => {
                self.registers.c = self.fetch_byte();
                self.clock(1, 2);
            },

            0x0F => {
                self.registers.a = self.alu_rrc(oldregs.a);
                self.registers.flag(Z, false);
                self.clock(1, 1);
            },

            0x11 => {
                let v = self.fetch_word();
                self.registers.set_de(v);
                self.clock(1, 3);
            },

            0x12 => {
                self.mmu.wb(self.registers.de(), self.registers.a);
                self.clock(1, 2);
            },

            0x13 => {
                let v = self.registers.de().wrapping_add(1);
                self.registers.set_de(v);
                self.clock(1, 2);
            },

            0x14 => {
                self.registers.d = self.alu_inc(oldregs.d);
                self.clock(1, 1);
            },

            0x15 => {
                self.registers.d = self.alu_dec(oldregs.d);
                self.clock(1, 1);
            },

            0x16 => {
                self.registers.d = self.fetch_byte();
                self.clock(1, 2);
            },

            0x17 => {
                self.registers.a = self.alu_rl(oldregs.a);
                self.registers.flag(Z, false);
                self.clock(1, 1);
            },

            0x18 => {
                self.cpu_jr();
                self.clock(1, 3);
            },

            0x19 => {
                let v = self.registers.de();
                self.alu_add16(v);
                self.clock(1, 2);
            },

            0x1A => {
                self.registers.a = self.mmu.rb(self.registers.de());
                self.clock(1, 2);
            },

            0x1B => {
                let v = self.registers.de().wrapping_sub(1);
                self.registers.set_de(v);
                self.clock(1, 2);
            },

            0x1C => {
                self.registers.e = self.alu_inc(oldregs.e);
                self.clock(1, 1);
            },

            0x1D => {
                self.registers.e = self.alu_dec(oldregs.e);
                self.clock(1, 1);
            },

            0x1E => {
                self.registers.e = self.fetch_byte();
                self.clock(1, 2);
            },

            0x1F => {
                self.registers.a = self.alu_rr(oldregs.a);
                self.registers.flag(Z, false);
                self.clock(1, 1);
            },

            0x20 => {
                if !self.registers.get_flag(Z) {
                    self.cpu_jr();
                    self.clock(1, 3);
                } else {
                    self.registers.pc += 1;
                    self.clock(1, 2);
                }
            },

            0x21 => {
                let v = self.fetch_word();
                self.registers.set_hl(v);
                self.clock(1, 3);
            },

            0x22 => {
                self.mmu.wb(self.registers.hli(), self.registers.a);
                self.clock(1, 2);
            },

            0x23 => {
                let v = self.registers.hl().wrapping_add(1);
                self.registers.set_hl(v);
                self.clock(1, 2);
            },

            0x24 => {
                self.registers.h = self.alu_inc(oldregs.h);
                self.clock(1, 1);
            },

            0x25 => {
                self.registers.h = self.alu_dec(oldregs.h);
                self.clock(1, 1);
            },

            0x26 => {
                self.registers.h = self.fetch_byte();
                self.clock(1, 2);
            },

            0x27 => {
                self.alu_daa();
                self.clock(1, 1);
            },

            0x28 => {
                if self.registers.get_flag(Z) {
                    self.cpu_jr();
                    self.clock(1, 3);
                } else {
                    self.registers.pc += 1;
                    self.clock(1, 2);
                }
            },

            0x29 => {
                let v = self.registers.hl();
                self.alu_add16(v);
                self.clock(1, 2);
            },

            0x2A => {
                self.registers.a = self.mmu.rb(self.registers.hli());
                self.clock(1, 2);
            },

            0x2B => {
                let v = self.registers.hl().wrapping_sub(1);
                self.registers.set_hl(v);
                self.clock(1, 2);
            },

            0x2C => {
                self.registers.l = self.alu_inc(oldregs.l);
                self.clock(1, 1);
            },

            0x2D => {
                self.registers.l = self.alu_dec(oldregs.l);
                self.clock(1, 1);
            },

            0x2E => {
                self.registers.l = self.fetch_byte();
                self.clock(1, 2);
            },

            0x2F => {
                self.registers.a = !self.registers.a;
                self.registers.flag(H, true);
                self.registers.flag(N, true);
                self.clock(1, 1);
            },

            0x30 => {
                if !self.registers.get_flag(C) {
                    self.cpu_jr();
                    self.clock(1, 3);
                } else {
                    self.registers.pc += 1;
                    self.clock(1, 2);
                }
            },

            0x31 => {
                self.registers.sp = self.fetch_word();
                self.clock(1, 3);
            },

            0x32 => {
                self.mmu.wb(self.registers.hld(), self.registers.a);
                self.clock(1, 2);
            },

            0x33 => {
                self.registers.sp = self.registers.sp.wrapping_add(1);
                self.clock(1, 2);
            },

            0x34 => {
                let a = self.registers.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_inc(v);
                self.mmu.wb(a, v2);
                self.clock(1, 3);
            },

            0x35 => {
                let a = self.registers.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_dec(v);
                self.mmu.wb(a, v2);
                self.clock(1, 3);
            },

            0x36 => {
                let v = self.fetch_byte();
                self.mmu.wb(self.registers.hl(), v);
                self.clock(1, 3);
            },

            0x37 => {
                self.registers.flag(C, true);
                self.registers.flag(H, false);
                self.registers.flag(N, false);
                self.clock(1, 1);
            },

            0x38 => {
                if self.registers.get_flag(C) {
                    self.cpu_jr();
                    self.clock(1, 3);
                } else {
                    self.registers.pc += 1;
                    self.clock(1, 2);
                }
            },

            0x39 => {
                let v = self.registers.sp;
                self.alu_add16(v);
                self.clock(1, 2);
            },

            0x3A => {
                self.registers.a = self.mmu.rb(self.registers.hld());
                self.clock(1, 2);
            },

            0x3B => {
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                self.clock(1, 2);
            },

            0x3C => {
                self.registers.a = self.alu_inc(oldregs.a);
                self.clock(1, 1);
            },

            0x3D => {
                self.registers.a = self.alu_dec(oldregs.a);
                self.clock(1, 1);
            },

            0x3E => {
                self.registers.a = self.fetch_byte();
                self.clock(1, 2);
            },

            0x3F => {
                let v = !self.registers.get_flag(C);
                self.registers.flag(C, v);
                self.registers.flag(H, false);
                self.registers.flag(N, false);
                self.clock(1, 1);
            },

            0x40 => {
                self.clock(1, 1);
            },

            0x41 => {
                self.registers.b = self.registers.c;
                self.clock(1, 1);
            },

            0x42 => {
                self.registers.b = self.registers.d;
                self.clock(1, 1);
            },

            0x43 => {
                self.registers.b = self.registers.e;
                self.clock(1, 1);
            },

            
            0x44 => {
                self.registers.b = self.registers.h;
                self.clock(1, 1);
            },

            
            0x45 => {
                self.registers.b = self.registers.l;
                self.clock(1, 1);
            },

            
            0x46 => {
                self.registers.b = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            
            0x47 => {
                self.registers.b = self.registers.a;
                self.clock(1, 1);
            },

            
            0x48 => {
                self.registers.c = self.registers.b;
                self.clock(1, 1);
            },

            
            0x49 => {
                self.clock(1, 1);
            },

            
            0x4A => {
                self.registers.c = self.registers.d;
                self.clock(1, 1);
            },

            
            0x4B => {
                self.registers.c = self.registers.e;
                self.clock(1, 1);
            },

            
            0x4C => {
                self.registers.c = self.registers.h;
                self.clock(1, 1);
            },

            
            0x4D => {
                self.registers.c = self.registers.l;
                self.clock(1, 1);
            },

            
            0x4E => {
                self.registers.c = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            
            0x4F => {
                self.registers.c = self.registers.a;
                self.clock(1, 1);
            },

            
            0x50 => {
                self.registers.d = self.registers.b;
                self.clock(1, 1);
            },

            
            0x51 => {
                self.registers.d = self.registers.c;
                self.clock(1, 1);
            },

            
            0x52 => {
                self.clock(1, 1);
            },

            
            0x53 => {
                self.registers.d = self.registers.e;
                self.clock(1, 1);
            },

            
            0x54 => {
                self.registers.d = self.registers.h;
                self.clock(1, 1);
            },

            
            0x55 => {
                self.registers.d = self.registers.l;
                self.clock(1, 1);
            },

            
            0x56 => {
                self.registers.d = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            
            0x57 => {
                self.registers.d = self.registers.a;
                self.clock(1, 1);
            },

            
            0x58 => {
                self.registers.e = self.registers.b;
                self.clock(1, 1);
            },

            
            0x59 => {
                self.registers.e = self.registers.c;
                self.clock(1, 1);
            },

            
            0x5A => {
                self.registers.e = self.registers.d;
                self.clock(1, 1);
            },

            
            0x5B => {
                self.clock(1, 1);
            },

            
            0x5C => {
                self.registers.e = self.registers.h;
                self.clock(1, 1);
            },

            
            0x5D => {
                self.registers.e = self.registers.l;
                self.clock(1, 1);
            },

            
            0x5E => {
                self.registers.e = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            
            0x5F => {
                self.registers.e = self.registers.a;
                self.clock(1, 1);
            },

            
            0x60 => {
                self.registers.h = self.registers.b;
                self.clock(1, 1);
            },

            
            0x61 => {
                self.registers.h = self.registers.c;
                self.clock(1, 1);
            },

            
            0x62 => {
                self.registers.h = self.registers.d;
                self.clock(1, 1);
            },

            
            0x63 => {
                self.registers.h = self.registers.e;
                self.clock(1, 1);
            },

            
            0x64 => {
                self.clock(1, 1);
            },

            
            0x65 => {
                self.registers.h = self.registers.l;
                self.clock(1, 1);
            },

            
            0x66 => {
                self.registers.h = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            
            0x67 => {
                self.registers.h = self.registers.a;
                self.clock(1, 1);
            },

            
            0x68 => {
                self.registers.l = self.registers.b;
                self.clock(1, 1);
            },

            
            0x69 => {
                self.registers.l = self.registers.c;
                self.clock(1, 1);
            },

            
            0x6A => {
                self.registers.l = self.registers.d;
                self.clock(1, 1);
            },

            
            0x6B => {
                self.registers.l = self.registers.e;
                self.clock(1, 1);
            },

            
            0x6C => {
                self.registers.l = self.registers.h;
                self.clock(1, 1);
            },

            
            0x6D => {
                self.clock(1, 1);
            },

            
            0x6E => {
                self.registers.l = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            
            0x6F => {
                self.registers.l = self.registers.a;
                self.clock(1, 1);
            },

            
            0x70 => {
                self.mmu.wb(self.registers.hl(), self.registers.b);
                self.clock(1, 2);
            },

            
            0x71 => {
                self.mmu.wb(self.registers.hl(), self.registers.c);
                self.clock(1, 2);
            },

            
            0x72 => {
                self.mmu.wb(self.registers.hl(), self.registers.d);
                self.clock(1, 2);
            },

            0x73 => {
                self.mmu.wb(self.registers.hl(), self.registers.e);
                self.clock(1, 2);
            },

            0x74 => {
                self.mmu.wb(self.registers.hl(), self.registers.h);
                self.clock(1, 2);
            },

            0x75 => {
                self.mmu.wb(self.registers.hl(), self.registers.l);
                self.clock(1, 2);
            },

            0x76 => {
                self.halted = true;
                self.clock(1, 1);
            },

            0x77 => {
                self.mmu.wb(self.registers.hl(), self.registers.a);
                self.clock(1, 2);
            },

            0x78 => {
                self.registers.a = self.registers.b;
                self.clock(1, 1);
            },

            0x79 => {
                self.registers.a = self.registers.c;
                self.clock(1, 1);
            },

            0x7A => {
                self.registers.a = self.registers.d;
                self.clock(1, 1);
            },

            0x7B => {
                self.registers.a = self.registers.e;
                self.clock(1, 1);
            },

            0x7C => {
                self.registers.a = self.registers.h;
                self.clock(1, 1);
            },

            0x7D => {
                self.registers.a = self.registers.l;
                self.clock(1, 1);
            },

            0x7E => {
                self.registers.a = self.mmu.rb(self.registers.hl());
                self.clock(1, 2);
            },

            0x7F => {
                self.clock(1, 1);
            },

            0x80 => {
                self.alu_add(oldregs.b, false);
                self.clock(1, 1);
            },

            0x81 => {
                self.alu_add(oldregs.c, false);
                self.clock(1, 1);
            },

            0x82 => {
                self.alu_add(oldregs.d, false);
                self.clock(1, 1);
            },

            0x83 => {
                self.alu_add(oldregs.e, false);
                self.clock(1, 1);
            },

            0x84 => {
                self.alu_add(oldregs.h, false);
                self.clock(1, 1);
            },

            0x85 => {
                self.alu_add(oldregs.l, false);
                self.clock(1, 1);
            },

            0x86 => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_add(v, false);
                self.clock(1, 2);
            },

            0x87 => {
                self.alu_add(oldregs.a, false);
                self.clock(1, 1);
            },

            0x88 => {
                self.alu_add(oldregs.b, true);
                self.clock(1, 1);
            },

            0x89 => {
                self.alu_add(oldregs.c, true);
                self.clock(1, 1);
            },

            0x8A => {
                self.alu_add(oldregs.d, true);
                self.clock(1, 1);
            },

            0x8B => {
                self.alu_add(oldregs.e, true);
                self.clock(1, 1);
            },

            0x8C => {
                self.alu_add(oldregs.h, true);
                self.clock(1, 1);
            },

            0x8D => {
                self.alu_add(oldregs.l, true);
                self.clock(1, 1);
            },

            0x8E => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_add(v, true);
                self.clock(1, 2);
            },

            0x8F => {
                self.alu_add(oldregs.a, true);
                self.clock(1, 1);
            },

            0x90 => {
                self.alu_sub(oldregs.b, false);
                self.clock(1, 1);
            },

            0x91 => {
                self.alu_sub(oldregs.c, false);
                self.clock(1, 1);
            },

            0x92 => {
                self.alu_sub(oldregs.d, false);
                self.clock(1, 1);
            },

            0x93 => {
                self.alu_sub(oldregs.e, false);
                self.clock(1, 1);
            },

            0x94 => {
                self.alu_sub(oldregs.h, false);
                self.clock(1, 1);
            },

            0x95 => {
                self.alu_sub(oldregs.l, false);
                self.clock(1, 1);
            },

            0x96 => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_sub(v, false);
                self.clock(1, 2);
            },

            0x97 => {
                self.alu_sub(oldregs.a, false);
                self.clock(1, 1);
            },

            0x98 => {
                self.alu_sub(oldregs.b, true);
                self.clock(1, 1);
            },

            0x99 => {
                self.alu_sub(oldregs.c, true);
                self.clock(1, 1);
            },

            0x9A => {
                self.alu_sub(oldregs.d, true);
                self.clock(1, 1);
            },

            0x9B => {
                self.alu_sub(oldregs.e, true);
                self.clock(1, 1);
            },

            0x9C => {
                self.alu_sub(oldregs.h, true);
                self.clock(1, 1);
            },

            0x9D => {
                self.alu_sub(oldregs.l, true);
                self.clock(1, 1);
            },

            0x9E => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_sub(v, true);
                self.clock(1, 2);
            },

            0x9F => {
                self.alu_sub(oldregs.a, true);
                self.clock(1, 1);
            },

            0xA0 => {
                self.alu_and(oldregs.b);
                self.clock(1, 1);
            },

            0xA1 => {
                self.alu_and(oldregs.c);
                self.clock(1, 1);
            },

            0xA2 => {
                self.alu_and(oldregs.d);
                self.clock(1, 1);
            },

            0xA3 => {
                self.alu_and(oldregs.e);
                self.clock(1, 1);
            },

            0xA4 => {
                self.alu_and(oldregs.h);
                self.clock(1, 1);
            },

            0xA5 => {
                self.alu_and(oldregs.l);
                self.clock(1, 1);
            },

            0xA6 => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_and(v);
                self.clock(1, 2);
            },

            0xA7 => {
                self.alu_and(oldregs.a);
                self.clock(1, 1);
            },

            0xA8 => {
                self.alu_xor(oldregs.b);
                self.clock(1, 1);
            },

            0xA9 => {
                self.alu_xor(oldregs.c);
                self.clock(1, 1);
            },

            0xAA => {
                self.alu_xor(oldregs.d);
                self.clock(1, 1);
            },

            0xAB => {
                self.alu_xor(oldregs.e);
                self.clock(1, 1);
            },

            0xAC => {
                self.alu_xor(oldregs.h);
                self.clock(1, 1);
            },

            0xAD => {
                self.alu_xor(oldregs.l);
                self.clock(1, 1);
            },

            0xAE => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_xor(v);
                self.clock(1, 2);
            },

            0xAF => {
                self.alu_xor(oldregs.a);
                self.clock(1, 1);
            },

            0xB0 => {
                self.alu_or(oldregs.b);
                self.clock(1, 1);
            },

            0xB1 => {
                self.alu_or(oldregs.c);
                self.clock(1, 1);
            },

            0xB2 => {
                self.alu_or(oldregs.d);
                self.clock(1, 1);
            },

            0xB3 => {
                self.alu_or(oldregs.e);
                self.clock(1, 1);
            },

            0xB4 => {
                self.alu_or(oldregs.h);
                self.clock(1, 1);
            },

            0xB5 => {
                self.alu_or(oldregs.l);
                self.clock(1, 1);
            },

            0xB6 => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_or(v);
                self.clock(1, 2);
            },

            0xB7 => {
                self.alu_or(oldregs.a);
                self.clock(1, 1);
            },

            0xB8 => {
                self.alu_cp(oldregs.b);
                self.clock(1, 1);
            },

            0xB9 => {
                self.alu_cp(oldregs.c);
                self.clock(1, 1);
            },

            0xBA => {
                self.alu_cp(oldregs.d);
                self.clock(1, 1);
            },

            0xBB => {
                self.alu_cp(oldregs.e);
                self.clock(1, 1);
            },

            0xBC => {
                self.alu_cp(oldregs.h);
                self.clock(1, 1);
            },

            0xBD => {
                self.alu_cp(oldregs.l);
                self.clock(1, 1);
            },

            0xBE => {
                let v = self.mmu.rb(self.registers.hl());
                self.alu_cp(v);
                self.clock(1, 2);
            },

            0xBF => {
                self.alu_cp(oldregs.a);
                self.clock(1, 1);
            },

            0xC0 => {
                if !self.registers.get_flag(Z) {
                    self.registers.pc = self.pop_stack();
                    self.clock(1, 5);
                } else {
                    self.clock(1, 2);
                }
            },

            0xC1 => {
                let v = self.pop_stack();
                self.registers.set_bc(v);
                self.clock(1, 3);
            },

            0xC2 => {
                if !self.registers.get_flag(Z) {
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 4);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xC3 => {
                self.registers.pc = self.fetch_word();
                self.clock(1, 4);
            },

            0xC4 => {
                if !self.registers.get_flag(Z) {
                    self.push_stack(oldregs.pc + 2);
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 6);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xC5 => {
                let v = self.registers.bc();
                self.push_stack(v);
                self.clock(1, 4);
            },

            0xC6 => {
                let v = self.fetch_byte();
                self.alu_add(v, false);
                self.clock(1, 2);
            },

            0xC7 => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x00;
                self.clock(1, 4);
            },

            0xC8 => {
                if self.registers.get_flag(Z) {
                    self.registers.pc = self.pop_stack();
                    self.clock(1, 5);
                } else {
                    self.clock(1, 2);
                }
            },

            0xC9 => {
                self.registers.pc = self.pop_stack();
                self.clock(1, 4);
            },

            0xCA => {
                if self.registers.get_flag(Z) {
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 4);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xCB => {
                //panic!("GameBoy color not implemented")
            },

            0xCC => {
                if self.registers.get_flag(Z) {
                    self.push_stack(oldregs.pc + 2);
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 6);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xCD => {
                self.push_stack(oldregs.pc + 2);
                self.registers.pc = self.fetch_word();
                self.clock(1, 6);
            },

            0xCE => {
                let v = self.fetch_byte();
                self.alu_add(v, true);
                self.clock(1, 2);
            },

            0xCF => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x08;
                self.clock(1, 4);
            },

            0xD0 => {
                if !self.registers.get_flag(C) {
                    self.registers.pc = self.pop_stack();
                    self.clock(1, 5);
                } else {
                    self.clock(1, 2);
                }
            },

            0xD1 => {
                let v = self.pop_stack();
                self.registers.set_de(v);
                self.clock(1, 3);
            },

            0xD2 => {
                if !self.registers.get_flag(C) {
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 4);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xD4 => {
                if !self.registers.get_flag(C) {
                    self.push_stack(oldregs.pc + 2);
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 6);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xD5 => {
                let v = self.registers.de();
                self.push_stack(v);
                self.clock(1, 4);
            },

            0xD6 => {
                let v = self.fetch_byte();
                self.alu_sub(v, false);
                self.clock(1, 2);
            },

            0xD7 => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x10;
                self.clock(1, 4);
            },

            0xD8 => {
                if self.registers.get_flag(C) {
                    self.registers.pc = self.pop_stack();
                    self.clock(1, 5);
                } else {
                    self.clock(1, 2);
                }
            },

            0xD9 => {
                self.registers.pc = self.pop_stack();
                self.setei = 1;
                self.clock(1, 4);
            },

            0xDA => {
                if self.registers.get_flag(C) {
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 4);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xDC => {
                if self.registers.get_flag(C) {
                    self.push_stack(oldregs.pc + 2);
                    self.registers.pc = self.fetch_word();
                    self.clock(1, 6);
                } else {
                    self.registers.pc += 2;
                    self.clock(1, 3);
                }
            },

            0xDE => {
                let v = self.fetch_byte();
                self.alu_sub(v, true);
                self.clock(1, 2);
            },

            0xDF => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x18;
                self.clock(1, 4);
            },

            0xE0 => {
                let a = 0xFF00 | self.fetch_byte() as u16;
                self.mmu.wb(a, self.registers.a);
                self.clock(1, 3);
            },

            0xE1 => {
                let v = self.pop_stack();
                self.registers.set_hl(v);
                self.clock(1, 3);
            },

            0xE2 => {
                self.mmu.wb(0xFF00 | self.registers.c as u16, self.registers.a);
                self.clock(1, 2);
            },

            0xE5 => {
                let v = self.registers.hl();
                self.push_stack(v);
                self.clock(1, 4);
            },

            0xE6 => {
                let v = self.fetch_byte();
                self.alu_and(v);
                self.clock(1, 2);
            },

            0xE7 => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x20;
                self.clock(1, 4);
            },

            0xE8 => {
                self.registers.sp = self.alu_add16imm(oldregs.sp);
                self.clock(1, 4);
            },

            0xE9 => {
                self.registers.pc = self.registers.hl();
                self.clock(1, 1);
            },

            0xEA => {
                let a = self.fetch_word();
                self.mmu.wb(a, self.registers.a);
                self.clock(1, 4);
            },

            0xEE => {
                let v = self.fetch_byte();
                self.alu_xor(v);
                self.clock(1, 2);
            },

            0xEF => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x28;
                self.clock(1, 4);
            },

            0xF0 => {
                let a = 0xFF00 | self.fetch_byte() as u16;
                self.registers.a = self.mmu.rb(a);
                self.clock(1, 3);
            },

            0xF1 => {
                let v = self.pop_stack() & 0xFFF0;
                self.registers.set_af(v);
                self.clock(1, 3);
            },

            0xF2 => {
                self.registers.a = self.mmu.rb(0xFF00 | self.registers.c as u16);
                self.clock(1, 2);
            },

            0xF3 => {
                self.setdi = 2;
                self.clock(1, 1);
            },

            0xF5 => {
                let v = self.registers.af();
                self.push_stack(v);
                self.clock(1, 4);
            },

            0xF6 => {
                let v = self.fetch_byte();
                self.alu_or(v);
                self.clock(1, 2);
            },

            0xF7 => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x30;
                self.clock(1, 4);
            },

            
            0xF8 => {
                let r = self.alu_add16imm(oldregs.sp);
                self.registers.set_hl(r);
                self.clock(1, 3);
            },

            
            0xF9 => {
                self.registers.sp = self.registers.hl();
                self.clock(1, 2);
            },

            
            0xFA => {
                let a = self.fetch_word();
                self.registers.a = self.mmu.rb(a);
                self.clock(1, 4);
            },

            
            0xFB => {
                self.setei = 2;
                self.clock(1, 1);
            },

            
            0xFE => {
                let v = self.fetch_byte();
                self.alu_cp(v);
                self.clock(1, 2);
            },

            
            0xFF => {
                self.push_stack(oldregs.pc);
                self.registers.pc = 0x38;
                self.clock(1, 4);
            },

            
            other => panic!("Instruction {:2X} is not implemented", other),
        }
    }

    fn alu_add(&mut self, b: u8, usec: bool) {
        let c = if usec && self.registers.get_flag(C) { 1 } else { 0 };
        let a = self.registers.a;
        let r = a.wrapping_add(b).wrapping_add(c);
        self.registers.flag(Z, r == 0);
        self.registers.flag(H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.registers.flag(N, false);
        self.registers.flag(C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.registers.a = r;
    }

    fn alu_sub(&mut self, b: u8, usec: bool) {
        let c = if usec && self.registers.get_flag(C) { 1 } else { 0 };
        let a = self.registers.a;
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.registers.flag(Z, r == 0);
        self.registers.flag(H, (a & 0x0F) < (b & 0x0F) + c);
        self.registers.flag(N, true);
        self.registers.flag(C, (a as u16) < (b as u16) + (c as u16));
        self.registers.a = r;
    }

    fn alu_and(&mut self, b: u8) {
        let r = self.registers.a & b;
        self.registers.flag(Z, r == 0);
        self.registers.flag(H, true);
        self.registers.flag(C, false);
        self.registers.flag(N, false);
        self.registers.a = r;
    }

    fn alu_or(&mut self, b: u8) {
        let r = self.registers.a | b;
        self.registers.flag(Z, r == 0);
        self.registers.flag(C, false);
        self.registers.flag(H, false);
        self.registers.flag(N, false);
        self.registers.a = r;
    }

    fn alu_xor(&mut self, b: u8) {
        let r = self.registers.a ^ b;
        self.registers.flag(Z, r == 0);
        self.registers.flag(C, false);
        self.registers.flag(H, false);
        self.registers.flag(N, false);
        self.registers.a = r;
    }

    fn alu_cp(&mut self, b: u8) {
        let r = self.registers.a;
        self.alu_sub(b, false);
        self.registers.a = r;
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.registers.flag(Z, r == 0);
        self.registers.flag(H, (a & 0x0F) + 1 > 0x0F);
        self.registers.flag(N, false);
        return r
    }

    fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.registers.flag(Z, r == 0);
        self.registers.flag(H, (a & 0x0F) == 0);
        self.registers.flag(N, true);
        return r
    }

    fn alu_add16(&mut self, b: u16) {
        let a = self.registers.hl();
        let r = a.wrapping_add(b);
        self.registers.flag(H, (a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.registers.flag(N, false);
        self.registers.flag(C, a > 0xFFFF - b);
        self.registers.set_hl(r);
    }

    fn alu_add16imm(&mut self, a: u16) -> u16 {
        let b = self.fetch_byte() as i8 as i16 as u16;
        self.registers.flag(N, false);
        self.registers.flag(Z, false);
        self.registers.flag(H, (a & 0x000F) + (b & 0x000F) > 0x000F);
        self.registers.flag(C, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);
        return a.wrapping_add(b)
    }

    fn alu_srflagupdate(&mut self, r: u8, c: bool) {
        self.registers.flag(H, false);
        self.registers.flag(N, false);
        self.registers.flag(Z, r == 0);
        self.registers.flag(C, c);
    }

    fn alu_rlc(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_rl(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.registers.get_flag(C) { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if self.registers.get_flag(C) { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_daa(&mut self) {
        let mut a = self.registers.a;
        let mut adjust = if self.registers.get_flag(C) { 0x60 } else { 0x00 };
        if self.registers.get_flag(H) { adjust |= 0x06; };
        if !self.registers.get_flag(N) {
            if a & 0x0F > 0x09 {
                adjust |= 0x06;
            };

            if a > 0x99 { adjust |= 0x60; };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.registers.flag(C, adjust >= 0x60);
        self.registers.flag(H, false);
        self.registers.flag(Z, a == 0);
        self.registers.a = a;
    }

    fn cpu_jr(&mut self) {
        let n = self.fetch_byte() as i8;
        self.registers.pc = ((self.registers.pc as u32 as i32) + (n as i32)) as u16;
    }

    pub fn get_gpu_pixels(&self) -> &[u8] {
        &self.mmu.gpu.data
    }


    pub fn key_down(&mut self, key: Key) {
        self.mmu.keypad.key_down(key);
    }

    pub fn key_up(&mut self, key: Key) {
        self.mmu.keypad.key_up(key);
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
        let mut cpu = Z80::new("./data/tetris.gb");
    }
}