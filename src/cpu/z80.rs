use cpu::registers::RegisterSet;
use memory::mmu::MMU;
use cpu::registers::CpuFlag::{C, N, H, Z};
use frontend::keypad::Key;

/// CPU Speed, set a 4194304 Hz (taken from the original hardware)
const CPU_SPEED: u32 = 4_194_304;

/// Z80 CPU
///
/// This is the brain of the GameBoy, where operations sent
/// by the game code are handled and it's internal state is changed
///
/// It provides a comprehensive access to the MMU and some operations
/// to control the state of the CPU (halted, etc.).
pub struct Z80 {
    /// Memory Management Unit
    ///
    /// This is where all game data is stored and manipulated, including
    /// the GPU, timer, keypad, etc. data. Opcodes call specific locations
    /// of this MMU in order to retrieve info or mutate it.
    ///
    /// More info in the module itself.
    mmu: MMU,

    /// Registers
    ///
    /// These are the main CPU register storage. These are a very fast
    /// storage location used by the CPU to perform basic operations
    /// with bits.
    ///
    /// More info the module itself.
    registers: RegisterSet,

    /// Halted flag
    ///
    /// This indicates whether the CPU is halted or not. It's set
    /// by an opcode, and only remains until an interrupt occurs (use for low battery)
    halted: bool,

    /// Interrupt master enable (INTERRUPT_MASTER_ENABLE)
    ///
    /// This is a internal variable we use to enable or disable
    /// interrupts that are defined in the MMU INTERRUPT_MASTER_ENABLE (0xFFF).
    /// 
    /// 0 -> Disable all interrupts
    /// 1 -> Enable interrupts from the IE in MMU
    interrupt_master_enable: bool,

    /// Set disable interrupt
    ///
    /// Whether we need to disable interrupts. This is set during
    /// an opcode and updates the CPU interrupt master enable flag
    /// in the next cycle of the CPU
    set_disable_interrupts: u32,
    
    /// Set enable interrupt
    /// 
    /// Inverse of the set disable interrupt. It has a different opcode
    set_enable_interrupts: u32,
    
    /// CPU internal clock
    /// 
    /// This is the main CPU clock, used to provide time limitation
    /// for opcodes in order to emulate original hardware
    clock: u32,

    /// CPU speed value
    ///
    /// Stores the CPU speed, which is calculated on initialization
    /// from the CPU_SPEED constant (taken from the original hardware)
    ///
    /// This is used to limit the FPS
    cpu_speed: u32
}

impl Z80 {
    pub fn new(rom_file: &str) -> Z80 {
        Z80 {
            registers: RegisterSet::new(),
            mmu: MMU::new(rom_file),
            halted: false,
            interrupt_master_enable: true,
            set_enable_interrupts: 0,
            set_disable_interrupts: 0,
            clock: 0,
            cpu_speed: ((CPU_SPEED / 1000) * 16) as u32
        }
    }

    /// Steps the CPU
    ///
    /// Notice that the clock ticks are taken from observation
    /// from the original CPU, and these are approximate, at exact
    /// cycle time is tied to special hardware constraints that
    /// are not normally reproduced in emulators (hence not-100% accuracy)
    pub fn step(&mut self) {
        while self.clock < self.cpu_speed {
            // cycle the CPU and obtain how much ticks
            // the operation took (used to limit the FPS)
            let op_clock = self.cycle();
            self.clock += op_clock;

            // steps the MMU, this will turn also steps in
            // GPU, keypad, timer, etc.
            self.mmu.step(op_clock);
        }

        // retract the clock by the same CPU
        // speed value, in order to keep cycling
        self.clock -= self.cpu_speed
    }

    fn cycle(&mut self) -> u32 {
        self.update_interrupt_master_enable();

        match self.interrupt() {
            0 => {
                0
            },
            n => {
                n
            },
        };

        if !self.halted {
            let opcode = self.read_byte();
            return self.execute(opcode);
        }

        1
    }

    fn update_interrupt_master_enable(&mut self) {
        self.set_disable_interrupts = match self.set_disable_interrupts {
            2 => 1,
            1 => {
                self.interrupt_master_enable = false;
                0
            },
            _ => 0,
        };

        self.set_enable_interrupts = match self.set_enable_interrupts {
            2 => 1,
            1 => {
                self.interrupt_master_enable = true;
                0
            },
            _ => 0,
        };
    }

    /// Handles interrupts
    ///
    /// This function checks for various interrupt sources (including MMU)
    /// in order to determine whether an interrupt ocurred.
    fn interrupt(&mut self) -> u32 {
        if self.interrupt_master_enable == false && self.halted == false {
            return 0
        }

        let triggered = self.mmu.interrupt_enable & self.mmu.interrupt_flag;

        if triggered == 0 {
            return 0
        }

        self.halted = false;
        self.interrupt_master_enable = false;

        // this stands for the interrupt beign triggered:
        // 0 -> VBlank
        // 1 -> GPU STAT
        // 2 -> Timer
        // 3 -> Serial Port
        // 4 -> Keypad
        let interrupt_number = triggered.trailing_zeros();

        if interrupt_number >= 5 {
            panic!("Invalid interrupt triggered");
        }

        self.mmu.interrupt_flag &= !(1 << interrupt_number);

        // get current program counter
        let program_counter = self.registers.program_counter;

        // push stack of the current program counter
        self.push_stack(program_counter);

        // point program counter to where the interrupt is handled
        // this is then fetch by the CPU and the opcode executed
        self.registers.program_counter = 0x0040 | ((interrupt_number as u16) << 3);

        // this operation takes 4 cycles
        return 4
    }

    fn push_stack(&mut self, value: u16) {
        self.registers.stack_pointer -= 2;

        self.mmu.write_word(
            self.registers.stack_pointer,
            value
        );
    }

    fn pop_stack(&mut self) -> u16 {
        let res = self.mmu.read_word(self.registers.stack_pointer);
        self.registers.stack_pointer += 2;
        res
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.mmu.read_byte(self.registers.program_counter);
        self.registers.program_counter
        += 1;
        b
    }

    fn read_word(&mut self) -> u16 {
        let w = self.mmu.read_word(self.registers.program_counter);
        self.registers.program_counter += 2;
        w
    }

    /// Executes an opcode
    ///
    /// This is where instructions sent by the game are handled.
    ///
    /// The Z80 contains 256 operations plus 256 CB-prefixed (see below)
    fn execute(&mut self, opcode: u8) -> u32 {
        let oldregs = self.registers;

        match opcode {
            0x00 => {
                1
            },

            0x01 => {
                let v = self.read_word();
                self.registers.set_bc(v);
                3
            },

            0x02 => {
                self.mmu.write_byte(self.registers.bc(), self.registers.a);
                2
            },

            0x03 => {
                let v = self.registers.bc().wrapping_add(1);
                self.registers.set_bc(v);
                2
            },

            0x04 => {
                self.registers.b = self.alu_increase(oldregs.b);
                1
            },

            0x05 => {
                self.registers.b = self.alu_decrease(oldregs.b);
                1
            },
            
            0x06 => {
                self.registers.b = self.read_byte();
                2
            },
            
            0x07 => {
                self.registers.a = self.alu_rlc(oldregs.a);
                self.registers.flag(Z, false);
                1
            },

            0x08 => {
                let a = self.read_word();
                self.mmu.write_word(a, self.registers.stack_pointer);
                5
            },

            0x09 => {
                let v = self.registers.bc();
                self.alu_add16(v);
                2
            },

            0x0A => {
                self.registers.a = self.mmu.read_byte(self.registers.bc());
                2
            },

            0x0B => {
                let v = self.registers.bc().wrapping_sub(1);
                self.registers.set_bc(v);
                2
            },

            0x0C => {
                self.registers.c = self.alu_increase(oldregs.c);
                1
            },

            0x0D => {
                self.registers.c = self.alu_decrease(oldregs.c);
                1
            },

            0x0E => {
                self.registers.c = self.read_byte();
                2
            },

            0x0F => {
                self.registers.a = self.alu_rrc(oldregs.a);
                self.registers.flag(Z, false);
                1
            },

            0x11 => {
                let v = self.read_word();
                self.registers.set_de(v);
                3
            },

            0x12 => {
                self.mmu.write_byte(self.registers.de(), self.registers.a);
                2
            },

            0x13 => {
                let v = self.registers.de().wrapping_add(1);
                self.registers.set_de(v);
                2
            },

            0x14 => {
                self.registers.d = self.alu_increase(oldregs.d);
                1
            },

            0x15 => {
                self.registers.d = self.alu_decrease(oldregs.d);
                1
            },

            0x16 => {
                self.registers.d = self.read_byte();
                2
            },

            0x17 => {
                self.registers.a = self.alu_rl(oldregs.a);
                self.registers.flag(Z, false);
                1
            },

            0x18 => {
                self.cpu_jr();
                3
            },

            0x19 => {
                let v = self.registers.de();
                self.alu_add16(v);
                2
            },

            0x1A => {
                self.registers.a = self.mmu.read_byte(self.registers.de());
                2
            },

            0x1B => {
                let v = self.registers.de().wrapping_sub(1);
                self.registers.set_de(v);
                2
            },

            0x1C => {
                self.registers.e = self.alu_increase(oldregs.e);
                1
            },

            0x1D => {
                self.registers.e = self.alu_decrease(oldregs.e);
                1
            },

            0x1E => {
                self.registers.e = self.read_byte();
                2
            },

            0x1F => {
                self.registers.a = self.alu_rr(oldregs.a);
                self.registers.flag(Z, false);
                1
            },

            0x20 => {
                if !self.registers.is_flag_set(Z) {
                    self.cpu_jr();
                    3
                } else {
                    self.registers.program_counter += 1;
                    2
                }
            },

            0x21 => {
                let v = self.read_word();
                self.registers.set_hl(v);
                3
            },

            0x22 => {
                self.mmu.write_byte(
                    self.registers.hl_increase(),
                    self.registers.a
                );

                2
            },

            0x23 => {
                let v = self.registers.hl().wrapping_add(1);
                self.registers.set_hl(v);
                2
            },

            0x24 => {
                self.registers.h = self.alu_increase(oldregs.h);
                1
            },

            0x25 => {
                self.registers.h = self.alu_decrease(oldregs.h);
                1
            },

            0x26 => {
                self.registers.h = self.read_byte();
                2
            },

            0x27 => {
                self.alu_daa();
                1
            },

            0x28 => {
                if self.registers.is_flag_set(Z) {
                    self.cpu_jr();
                    3
                } else {
                    self.registers.program_counter += 1;
                    2
                }
            },

            0x29 => {
                let v = self.registers.hl();
                self.alu_add16(v);
                2
            },

            0x2A => {
                self.registers.a = self.mmu.read_byte(self.registers.hl_increase());
                2
            },

            0x2B => {
                let v = self.registers.hl().wrapping_sub(1);
                self.registers.set_hl(v);
                2
            },

            0x2C => {
                self.registers.l = self.alu_increase(oldregs.l);
                1
            },

            0x2D => {
                self.registers.l = self.alu_decrease(oldregs.l);
                1
            },

            0x2E => {
                self.registers.l = self.read_byte();
                2
            },

            0x2F => {
                self.registers.a = !self.registers.a;
                self.registers.flag(H, true);
                self.registers.flag(N, true);
                1
            },

            0x30 => {
                if !self.registers.is_flag_set(C) {
                    self.cpu_jr();
                    3
                } else {
                    self.registers.program_counter += 1;
                    2
                }
            },

            0x31 => {
                self.registers.stack_pointer = self.read_word();
                3
            },

            0x32 => {
                self.mmu.write_byte(
                    self.registers.hl_decrease(),
                    self.registers.a
                );
                2
            },

            0x33 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
                2
            },

            0x34 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a);
                let v2 = self.alu_increase(v);
                self.mmu.write_byte(a, v2);
                3
            },

            0x35 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a);
                let v2 = self.alu_decrease(v);
                self.mmu.write_byte(a, v2);
                3
            },

            0x36 => {
                let v = self.read_byte();
                self.mmu.write_byte(self.registers.hl(), v);
                3
            },

            0x37 => {
                self.registers.flag(C, true);
                self.registers.flag(H, false);
                self.registers.flag(N, false);
                1
            },

            0x38 => {
                if self.registers.is_flag_set(C) {
                    self.cpu_jr();
                    3
                } else {
                    self.registers.program_counter += 1;
                    2
                }
            },

            0x39 => {
                let v = self.registers.stack_pointer;
                self.alu_add16(v);
                2
            },

            0x3A => {
                self.registers.a = self.mmu.read_byte(self.registers.hl_decrease());
                2
            },

            0x3B => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                2
            },

            0x3C => {
                self.registers.a = self.alu_increase(oldregs.a);
                1
            },

            0x3D => {
                self.registers.a = self.alu_decrease(oldregs.a);
                1
            },

            0x3E => {
                self.registers.a = self.read_byte();
                2
            },

            0x3F => {
                let v = !self.registers.is_flag_set(C);
                self.registers.flag(C, v);
                self.registers.flag(H, false);
                self.registers.flag(N, false);
                1
            },

            0x40 => {
                1
            },

            0x41 => {
                self.registers.b = self.registers.c;
                1
            },

            0x42 => {
                self.registers.b = self.registers.d;
                1
            },

            0x43 => {
                self.registers.b = self.registers.e;
                1
            },


            0x44 => {
                self.registers.b = self.registers.h;
                1
            },


            0x45 => {
                self.registers.b = self.registers.l;
                1
            },


            0x46 => {
                self.registers.b = self.mmu.read_byte(self.registers.hl());
                2
            },


            0x47 => {
                self.registers.b = self.registers.a;
                1
            },


            0x48 => {
                self.registers.c = self.registers.b;
                1
            },


            0x49 => {
                1
            },


            0x4A => {
                self.registers.c = self.registers.d;
                1
            },


            0x4B => {
                self.registers.c = self.registers.e;
                1
            },


            0x4C => {
                self.registers.c = self.registers.h;
                1
            },


            0x4D => {
                self.registers.c = self.registers.l;
                1
            },


            0x4E => {
                self.registers.c = self.mmu.read_byte(self.registers.hl());
                2
            },


            0x4F => {
                self.registers.c = self.registers.a;
                1
            },


            0x50 => {
                self.registers.d = self.registers.b;
                1
            },


            0x51 => {
                self.registers.d = self.registers.c;
                1
            },


            0x52 => {
                1
            },


            0x53 => {
                self.registers.d = self.registers.e;
                1
            },


            0x54 => {
                self.registers.d = self.registers.h;
                1
            },


            0x55 => {
                self.registers.d = self.registers.l;
                1
            },


            0x56 => {
                self.registers.d = self.mmu.read_byte(self.registers.hl());
                2
            },


            0x57 => {
                self.registers.d = self.registers.a;
                1
            },


            0x58 => {
                self.registers.e = self.registers.b;
                1
            },


            0x59 => {
                self.registers.e = self.registers.c;
                1
            },


            0x5A => {
                self.registers.e = self.registers.d;
                1
            },


            0x5B => {
                1
            },


            0x5C => {
                self.registers.e = self.registers.h;
                1
            },


            0x5D => {
                self.registers.e = self.registers.l;
                1
            },


            0x5E => {
                self.registers.e = self.mmu.read_byte(self.registers.hl());
                2
            },


            0x5F => {
                self.registers.e = self.registers.a;
                1
            },


            0x60 => {
                self.registers.h = self.registers.b;
                1
            },


            0x61 => {
                self.registers.h = self.registers.c;
                1
            },


            0x62 => {
                self.registers.h = self.registers.d;
                1
            },


            0x63 => {
                self.registers.h = self.registers.e;
                1
            },


            0x64 => {
                1
            },


            0x65 => {
                self.registers.h = self.registers.l;
                1
            },


            0x66 => {
                self.registers.h = self.mmu.read_byte(self.registers.hl());
                2
            },


            0x67 => {
                self.registers.h = self.registers.a;
                1
            },


            0x68 => {
                self.registers.l = self.registers.b;
                1
            },


            0x69 => {
                self.registers.l = self.registers.c;
                1
            },


            0x6A => {
                self.registers.l = self.registers.d;
                1
            },


            0x6B => {
                self.registers.l = self.registers.e;
                1
            },


            0x6C => {
                self.registers.l = self.registers.h;
                1
            },


            0x6D => {
                1
            },


            0x6E => {
                self.registers.l = self.mmu.read_byte(self.registers.hl());
                2
            },


            0x6F => {
                self.registers.l = self.registers.a;
                1
            },


            0x70 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.b);
                2
            },


            0x71 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.c);
                2
            },


            0x72 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.d);
                2
            },

            0x73 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.e);
                2
            },

            0x74 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.h);
                2
            },

            0x75 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.l);
                2
            },

            0x76 => {
                self.halted = true;
                1
            },

            0x77 => {
                self.mmu.write_byte(self.registers.hl(), self.registers.a);
                2
            },

            0x78 => {
                self.registers.a = self.registers.b;
                1
            },

            0x79 => {
                self.registers.a = self.registers.c;
                1
            },

            0x7A => {
                self.registers.a = self.registers.d;
                1
            },

            0x7B => {
                self.registers.a = self.registers.e;
                1
            },

            0x7C => {
                self.registers.a = self.registers.h;
                1
            },

            0x7D => {
                self.registers.a = self.registers.l;
                1
            },

            0x7E => {
                self.registers.a = self.mmu.read_byte(self.registers.hl());
                2
            },

            0x7F => {
                1
            },

            0x80 => {
                self.alu_add(oldregs.b, false);
                1
            },

            0x81 => {
                self.alu_add(oldregs.c, false);
                1
            },

            0x82 => {
                self.alu_add(oldregs.d, false);
                1
            },

            0x83 => {
                self.alu_add(oldregs.e, false);
                1
            },

            0x84 => {
                self.alu_add(oldregs.h, false);
                1
            },

            0x85 => {
                self.alu_add(oldregs.l, false);
                1
            },

            0x86 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_add(v, false);
                2
            },

            0x87 => {
                self.alu_add(oldregs.a, false);
                1
            },

            0x88 => {
                self.alu_add(oldregs.b, true);
                1
            },

            0x89 => {
                self.alu_add(oldregs.c, true);
                1
            },

            0x8A => {
                self.alu_add(oldregs.d, true);
                1
            },

            0x8B => {
                self.alu_add(oldregs.e, true);
                1
            },

            0x8C => {
                self.alu_add(oldregs.h, true);
                1
            },

            0x8D => {
                self.alu_add(oldregs.l, true);
                1
            },

            0x8E => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_add(v, true);
                2
            },

            0x8F => {
                self.alu_add(oldregs.a, true);
                1
            },

            0x90 => {
                self.alu_subtract(oldregs.b, false);
                1
            },

            0x91 => {
                self.alu_subtract(oldregs.c, false);
                1
            },

            0x92 => {
                self.alu_subtract(oldregs.d, false);
                1
            },

            0x93 => {
                self.alu_subtract(oldregs.e, false);
                1
            },

            0x94 => {
                self.alu_subtract(oldregs.h, false);
                1
            },

            0x95 => {
                self.alu_subtract(oldregs.l, false);
                1
            },

            0x96 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_subtract(v, false);
                2
            },

            0x97 => {
                self.alu_subtract(oldregs.a, false);
                1
            },

            0x98 => {
                self.alu_subtract(oldregs.b, true);
                1
            },

            0x99 => {
                self.alu_subtract(oldregs.c, true);
                1
            },

            0x9A => {
                self.alu_subtract(oldregs.d, true);
                1
            },

            0x9B => {
                self.alu_subtract(oldregs.e, true);
                1
            },

            0x9C => {
                self.alu_subtract(oldregs.h, true);
                1
            },

            0x9D => {
                self.alu_subtract(oldregs.l, true);
                1
            },

            0x9E => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_subtract(v, true);
                2
            },

            0x9F => {
                self.alu_subtract(oldregs.a, true);
                1
            },

            0xA0 => {
                self.alu_and(oldregs.b);
                1
            },

            0xA1 => {
                self.alu_and(oldregs.c);
                1
            },

            0xA2 => {
                self.alu_and(oldregs.d);
                1
            },

            0xA3 => {
                self.alu_and(oldregs.e);
                1
            },

            0xA4 => {
                self.alu_and(oldregs.h);
                1
            },

            0xA5 => {
                self.alu_and(oldregs.l);
                1
            },

            0xA6 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_and(v);
                2
            },

            0xA7 => {
                self.alu_and(oldregs.a);
                1
            },

            0xA8 => {
                self.alu_xor(oldregs.b);
                1
            },

            0xA9 => {
                self.alu_xor(oldregs.c);
                1
            },

            0xAA => {
                self.alu_xor(oldregs.d);
                1
            },

            0xAB => {
                self.alu_xor(oldregs.e);
                1
            },

            0xAC => {
                self.alu_xor(oldregs.h);
                1
            },

            0xAD => {
                self.alu_xor(oldregs.l);
                1
            },

            0xAE => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_xor(v);
                2
            },

            0xAF => {
                self.alu_xor(oldregs.a);
                1
            },

            0xB0 => {
                self.alu_or(oldregs.b);
                1
            },

            0xB1 => {
                self.alu_or(oldregs.c);
                1
            },

            0xB2 => {
                self.alu_or(oldregs.d);
                1
            },

            0xB3 => {
                self.alu_or(oldregs.e);
                1
            },

            0xB4 => {
                self.alu_or(oldregs.h);
                1
            },

            0xB5 => {
                self.alu_or(oldregs.l);
                1
            },

            0xB6 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_or(v);
                2
            },

            0xB7 => {
                self.alu_or(oldregs.a);
                1
            },

            0xB8 => {
                self.alu_compare(oldregs.b);
                1
            },

            0xB9 => {
                self.alu_compare(oldregs.c);
                1
            },

            0xBA => {
                self.alu_compare(oldregs.d);
                1
            },

            0xBB => {
                self.alu_compare(oldregs.e);
                1
            },

            0xBC => {
                self.alu_compare(oldregs.h);
                1
            },

            0xBD => {
                self.alu_compare(oldregs.l);
                1
            },

            0xBE => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_compare(v);
                2
            },

            0xBF => {
                self.alu_compare(oldregs.a);
                1
            },

            0xC0 => {
                if !self.registers.is_flag_set(Z) {
                    self.registers.program_counter = self.pop_stack();
                    5
                } else {
                    2
                }
            },

            0xC1 => {
                let v = self.pop_stack();
                self.registers.set_bc(v);
                3
            },

            0xC2 => {
                if !self.registers.is_flag_set(Z) {
                    self.registers.program_counter = self.read_word();
                    4
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            0xC3 => {
                self.registers.program_counter = self.read_word();
                4
            },

            0xC4 => {
                if !self.registers.is_flag_set(Z) {
                    self.push_stack(oldregs.program_counter + 2);
                    self.registers.program_counter = self.read_word();
                    6
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            0xC5 => {
                let v = self.registers.bc();
                self.push_stack(v);
                4
            },

            0xC6 => {
                let v = self.read_byte();
                self.alu_add(v, false);
                2
            },

            0xC7 => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x00;
                4
            },

            0xC8 => {
                if self.registers.is_flag_set(Z) {
                    self.registers.program_counter = self.pop_stack();
                    5
                } else {
                    2
                }
            },

            0xC9 => {
                self.registers.program_counter = self.pop_stack();
                4
            },

            0xCA => {
                if self.registers.is_flag_set(Z) {
                    self.registers.program_counter = self.read_word();
                    4
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            // CB-prefixed operations, call a different set
            // of operations (see method for more info)
            0xCB => {
                self.execute_cb(oldregs)
            },

            0xCC => {
                if self.registers.is_flag_set(Z) {
                    self.push_stack(oldregs.program_counter + 2);
                    self.registers.program_counter = self.read_word();
                    6
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            // Commented example of stack work, not using
            // code functions for clarity
            0xCD => {
                // decrease current stack pointer to the current function
                self.registers.stack_pointer -= 2;

                // write address of the current instruction forward
                self.mmu.write_word(
                    self.registers.stack_pointer,
                    oldregs.program_counter + 2
                );

                // point the program counter to the current function
                self.registers.program_counter = self.read_word();

                6
            },

            0xCE => {
                let v = self.read_byte();
                self.alu_add(v, true);
                2
            },

            0xCF => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x08;
                4
            },

            0xD0 => {
                if !self.registers.is_flag_set(C) {
                    self.registers.program_counter = self.pop_stack();
                    5
                } else {
                    2
                }
            },

            0xD1 => {
                let v = self.pop_stack();
                self.registers.set_de(v);
                3
            },

            0xD2 => {
                if !self.registers.is_flag_set(C) {
                    self.registers.program_counter = self.read_word();
                    4
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            0xD4 => {
                if !self.registers.is_flag_set(C) {
                    self.push_stack(oldregs.program_counter + 2);
                    self.registers.program_counter = self.read_word();
                    6
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            0xD5 => {
                let v = self.registers.de();
                self.push_stack(v);
                4
            },

            0xD6 => {
                let v = self.read_byte();
                self.alu_subtract(v, false);
                2
            },

            0xD7 => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x10;
                4
            },

            0xD8 => {
                if self.registers.is_flag_set(C) {
                    self.registers.program_counter = self.pop_stack();
                    5
                } else {
                    2
                }
            },

            0xD9 => {
                self.registers.program_counter = self.pop_stack();
                self.set_enable_interrupts = 1;
                4
            },

            0xDA => {
                if self.registers.is_flag_set(C) {
                    self.registers.program_counter = self.read_word();
                    4
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            0xDC => {
                if self.registers.is_flag_set(C) {
                    self.push_stack(oldregs.program_counter + 2);
                    self.registers.program_counter = self.read_word();
                    6
                } else {
                    self.registers.program_counter += 2;
                    3
                }
            },

            0xDE => {
                let v = self.read_byte();
                self.alu_subtract(v, true);
                2
            },

            0xDF => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x18;
                4
            },

            0xE0 => {
                let a = 0xFF00 | self.read_byte() as u16;
                self.mmu.write_byte(a, self.registers.a);
                3
            },

            0xE1 => {
                let v = self.pop_stack();
                self.registers.set_hl(v);
                3
            },

            0xE2 => {
                self.mmu.write_byte(0xFF00 | self.registers.c as u16, self.registers.a);
                2
            },

            0xE5 => {
                let v = self.registers.hl();
                self.push_stack(v);
                4
            },

            0xE6 => {
                let v = self.read_byte();
                self.alu_and(v);
                2
            },

            0xE7 => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x20;
                4
            },

            0xE8 => {
                self.registers.stack_pointer = self.alu_add16imm(oldregs.stack_pointer);
                4
            },

            0xE9 => {
                self.registers.program_counter = self.registers.hl();
                1
            },

            0xEA => {
                let a = self.read_word();
                self.mmu.write_byte(a, self.registers.a);
                4
            },

            0xEE => {
                let v = self.read_byte();
                self.alu_xor(v);
                2
            },

            0xEF => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x28;
                4
            },

            0xF0 => {
                let a = 0xFF00 | self.read_byte() as u16;
                self.registers.a = self.mmu.read_byte(a);
                3
            },

            0xF1 => {
                let v = self.pop_stack() & 0xFFF0;
                self.registers.set_af(v);
                3
            },

            0xF2 => {
                self.registers.a = self.mmu.read_byte(0xFF00 | self.registers.c as u16);
                2
            },

            0xF3 => {
                self.set_disable_interrupts = 2;
                1
            },

            0xF5 => {
                let v = self.registers.af();
                self.push_stack(v);
                4
            },

            0xF6 => {
                let v = self.read_byte();
                self.alu_or(v);
                2
            },

            0xF7 => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x30;
                4
            },


            0xF8 => {
                let r = self.alu_add16imm(oldregs.stack_pointer);
                self.registers.set_hl(r);
                3
            },


            0xF9 => {
                self.registers.stack_pointer = self.registers.hl();
                2
            },


            0xFA => {
                let a = self.read_word();
                self.registers.a = self.mmu.read_byte(a);
                4
            },


            0xFB => {
                self.set_enable_interrupts = 2;
                1
            },


            0xFE => {
                let v = self.read_byte();
                self.alu_compare(v);
                2
            },


            0xFF => {
                self.push_stack(oldregs.program_counter);
                self.registers.program_counter = 0x38;
                4
            },

            other => panic!("CPU instruction not implemented: {:2X}", other),
        }
    }

    /// Execute CB-prefixed operations
    ///
    /// When an operation is CB-prefixed (CB is hex), these special
    /// operations are called. This is simply to allow the Z80 to handle
    /// a bigger number of operations
    fn execute_cb(&mut self, oldregs: RegisterSet) -> u32 {
        let opcode = self.read_byte();

        match opcode {
            // RLC B
            0x00 => {
                self.registers.b = self.alu_rlc(oldregs.b);
                2
            },

            // RLC C
            0x01 => {
                self.registers.c = self.alu_rlc(oldregs.c);
                2
            },

            // RLC D
            0x02 => {
                self.registers.d = self.alu_rlc(oldregs.d);
                2
            },

            0x03 => {
                self.registers.e = self.alu_rlc(oldregs.e);
                2
            },

            0x04 => {
                self.registers.h = self.alu_rlc(oldregs.h);
                2
            },

            0x05 => {
                self.registers.l = self.alu_rlc(oldregs.l);
                2
            },

            0x06 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a);
                let v2 = self.alu_rlc(v);
                self.mmu.write_byte(a, v2);
                4
            },

            0x07 => {
                self.registers.a = self.alu_rlc(oldregs.a);
                2
            },

            0x08 => {
                self.registers.b = self.alu_rrc(oldregs.b);
                2
            },

            0x09 => {
                self.registers.c = self.alu_rrc(oldregs.c);
                2
            },

            0x0A => {
                self.registers.d = self.alu_rrc(oldregs.d);
                2
            },

            0x0B => {
                self.registers.e = self.alu_rrc(oldregs.e);
                2
            },

            0x0C => {
                self.registers.h = self.alu_rrc(oldregs.h);
                2
            },

            0x0D => {
                self.registers.l = self.alu_rrc(oldregs.l);
                2
            },

            0x0E => {
                let a = self.registers.hl(); let v = self.mmu.read_byte(a); let v2 = self.alu_rrc(v); self.mmu.write_byte(a, v2);
                4
            },

            0x0F => {
                self.registers.a = self.alu_rrc(oldregs.a);
                2
            },

            0x10 => {
                self.registers.b = self.alu_rl(oldregs.b);
                2
            },

            0x11 => {
                self.registers.c = self.alu_rl(oldregs.c);
                2
            },

            0x12 => {
                self.registers.d = self.alu_rl(oldregs.d);
                2
            },

            0x13 => {
                self.registers.e = self.alu_rl(oldregs.e);
                2
            },

            0x14 => {
                self.registers.h = self.alu_rl(oldregs.h);
                2
            },

            0x15 => {
                self.registers.l = self.alu_rl(oldregs.l);
                2
            },

            0x16 => {
                let a = self.registers.hl(); let v = self.mmu.read_byte(a); let v2 = self.alu_rl(v); self.mmu.write_byte(a, v2);
                4
            },

            0x17 => {
                self.registers.a = self.alu_rl(oldregs.a);
                2
            },

            0x18 => {
                self.registers.b = self.alu_rr(oldregs.b);
                2
            },

            0x19 => {
                self.registers.c = self.alu_rr(oldregs.c);
                2
            },

            0x1A => {
                self.registers.d = self.alu_rr(oldregs.d);
                2
            },

            0x1B => {
                self.registers.e = self.alu_rr(oldregs.e);
                2
            },

            0x1C => {
                self.registers.h = self.alu_rr(oldregs.h);
                2
            },

            0x1D => {
                self.registers.l = self.alu_rr(oldregs.l);
                2
            },

            0x1E => {
                let a = self.registers.hl(); let v = self.mmu.read_byte(a); let v2 = self.alu_rr(v); self.mmu.write_byte(a, v2);
                4
            },

            0x1F => {
                self.registers.a = self.alu_rr(oldregs.a);
                2
            },

            0x20 => {
                self.registers.b = self.alu_sla(oldregs.b);
                2
            },

            0x21 => {
                self.registers.c = self.alu_sla(oldregs.c);
                2
            },

            0x22 => {
                self.registers.d = self.alu_sla(oldregs.d);
                2
            },

            0x23 => {
                self.registers.e = self.alu_sla(oldregs.e);
                2
            },

            0x24 => {
                self.registers.h = self.alu_sla(oldregs.h);
                2
            },

            0x25 => {
                self.registers.l = self.alu_sla(oldregs.l);
                2
            },

            0x26 => {
                let a = self.registers.hl(); let v = self.mmu.read_byte(a); let v2 = self.alu_sla(v); self.mmu.write_byte(a, v2);
                4
            },

            0x27 => {
                self.registers.a = self.alu_sla(oldregs.a);
                2
            },

            0x28 => {
                self.registers.b = self.alu_sra(oldregs.b);
                2
            },

            0x29 => {
                self.registers.c = self.alu_sra(oldregs.c);
                2
            },

            0x2A => {
                self.registers.d = self.alu_sra(oldregs.d);
                2
            },

            0x2B => {
                self.registers.e = self.alu_sra(oldregs.e);
                2
            },

            0x2C => {
                self.registers.h = self.alu_sra(oldregs.h);
                2
            },

            0x2D => {
                self.registers.l = self.alu_sra(oldregs.l);
                2
            },

            0x2E => {
                let a = self.registers.hl(); let v = self.mmu.read_byte(a); let v2 = self.alu_sra(v); self.mmu.write_byte(a, v2);
                4
            },

            0x2F => {
                self.registers.a = self.alu_sra(oldregs.a);
                2
            },

            0x30 => {
                self.registers.b = self.alu_swap(oldregs.b);
                2
            },

            0x31 => {
                self.registers.c = self.alu_swap(oldregs.c);
                2
            },

            0x32 => {
                self.registers.d = self.alu_swap(oldregs.d);
                2
            },

            0x33 => {
                self.registers.e = self.alu_swap(oldregs.e);
                2
            },

            0x34 => {
                self.registers.h = self.alu_swap(oldregs.h);
                2
            },

            0x35 => {
                self.registers.l = self.alu_swap(oldregs.l);
                2
            },

            0x36 => {
                let a = self.registers.hl(); let v = self.mmu.read_byte(a); let v2 = self.alu_swap(v); self.mmu.write_byte(a, v2);
                4
            },

            0x37 => {
                self.registers.a = self.alu_swap(oldregs.a);
                2
            },

            0x38 => {
                self.registers.b = self.alu_srl(oldregs.b);
                2
            },

            0x39 => {
                self.registers.c = self.alu_srl(oldregs.c);
                2
            },

            0x3A => {
                self.registers.d = self.alu_srl(oldregs.d);
                2
            },

            0x3B => {
                self.registers.e = self.alu_srl(oldregs.e);
                2
            },

            0x3C => {
                self.registers.h = self.alu_srl(oldregs.h);
                2
            },

            0x3D => {
                self.registers.l = self.alu_srl(oldregs.l);
                2
            },

            0x3E => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a);
                let v2 = self.alu_srl(v);
                self.mmu.write_byte(a, v2);

                4
            },

            0x3F => {
                self.registers.a = self.alu_srl(oldregs.a);
                2
            },

            0x40 => {
                self.alu_bit(oldregs.b, 0);
                2
            },

            0x41 => {
                self.alu_bit(oldregs.c, 0);
                2
            },

            0x42 => {
                self.alu_bit(oldregs.d, 0);
                2
            },

            0x43 => {
                self.alu_bit(oldregs.e, 0);
                2
            },

            0x44 => {
                self.alu_bit(oldregs.h, 0);
                2
            },

            0x45 => {
                self.alu_bit(oldregs.l, 0);
                2
            },

            0x46 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 0);
                3
            },

            0x47 => {
                self.alu_bit(oldregs.a, 0);
                2
            },

            0x48 => {
                self.alu_bit(oldregs.b, 1);
                2
            },

            0x49 => {
                self.alu_bit(oldregs.c, 1);
                2
            },

            0x4A => {
                self.alu_bit(oldregs.d, 1);
                2
            },

            0x4B => {
                self.alu_bit(oldregs.e, 1);
                2
            },

            0x4C => {
                self.alu_bit(oldregs.h, 1);
                2
            },

            0x4D => {
                self.alu_bit(oldregs.l, 1);
                2
            },

            0x4E => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 1);
                3
            },

            0x4F => {
                self.alu_bit(oldregs.a, 1);
                2
            },

            0x50 => {
                self.alu_bit(oldregs.b, 2);
                2
            },

            0x51 => {
                self.alu_bit(oldregs.c, 2);
                2
            },

            0x52 => {
                self.alu_bit(oldregs.d, 2);
                2
            },

            0x53 => {
                self.alu_bit(oldregs.e, 2);
                2
            },

            0x54 => {
                self.alu_bit(oldregs.h, 2);
                2
            },

            0x55 => {
                self.alu_bit(oldregs.l, 2);
                2
            },

            0x56 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 2);

                3
            },

            0x57 => {
                self.alu_bit(oldregs.a, 2);
                2
            },

            0x58 => {
                self.alu_bit(oldregs.b, 3);
                2
            },

            0x59 => {
                self.alu_bit(oldregs.c, 3);
                2
            },

            0x5A => {
                self.alu_bit(oldregs.d, 3);
                2
            },

            0x5B => {
                self.alu_bit(oldregs.e, 3);
                2
            },

            0x5C => {
                self.alu_bit(oldregs.h, 3);
                2
            },

            0x5D => {
                self.alu_bit(oldregs.l, 3);
                2
            },

            0x5E => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 3);

                3
            },

            0x5F => {
                self.alu_bit(oldregs.a, 3);
                2
            },

            0x60 => {
                self.alu_bit(oldregs.b, 4);
                2
            },

            0x61 => {
                self.alu_bit(oldregs.c, 4);
                2
            },

            0x62 => {
                self.alu_bit(oldregs.d, 4);
                2
            },

            0x63 => {
                self.alu_bit(oldregs.e, 4);
                2
            },

            0x64 => {
                self.alu_bit(oldregs.h, 4);
                2
            },

            0x65 => {
                self.alu_bit(oldregs.l, 4);
                2
            },

            0x66 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 4);

                3
            },

            0x67 => {
                self.alu_bit(oldregs.a, 4);
                2
            },

            0x68 => {
                self.alu_bit(oldregs.b, 5);
                2
            },

            0x69 => {
                self.alu_bit(oldregs.c, 5);
                2
            },

            0x6A => {
                self.alu_bit(oldregs.d, 5);
                2
            },

            0x6B => {
                self.alu_bit(oldregs.e, 5);
                2
            },

            0x6C => {
                self.alu_bit(oldregs.h, 5);
                2
            },

            0x6D => {
                self.alu_bit(oldregs.l, 5);
                2
            },

            0x6E => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 5);

                3
            },

            0x6F => {
                self.alu_bit(oldregs.a, 5);
                2
            },

            0x70 => {
                self.alu_bit(oldregs.b, 6);
                2
            },

            0x71 => {
                self.alu_bit(oldregs.c, 6);
                2
            },

            0x72 => {
                self.alu_bit(oldregs.d, 6);
                2
            },

            0x73 => {
                self.alu_bit(oldregs.e, 6);
                2
            },

            0x74 => {
                self.alu_bit(oldregs.h, 6);
                2
            },

            0x75 => {
                self.alu_bit(oldregs.l, 6);
                2
            },

            0x76 => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 6);

                3
            },

            0x77 => {
                self.alu_bit(oldregs.a, 6);
                2
            },

            0x78 => {
                self.alu_bit(oldregs.b, 7);
                2
            },

            0x79 => {
                self.alu_bit(oldregs.c, 7);
                2
            },

            0x7A => {
                self.alu_bit(oldregs.d, 7);
                2
            },

            0x7B => {
                self.alu_bit(oldregs.e, 7);
                2
            },

            0x7C => {
                self.alu_bit(oldregs.h, 7);
                2
            },

            0x7D => {
                self.alu_bit(oldregs.l, 7);
                2
            },

            0x7E => {
                let v = self.mmu.read_byte(self.registers.hl());
                self.alu_bit(v, 7);

                3
            },

            0x7F => {
                self.alu_bit(oldregs.a, 7);
                2
            },

            0x80 => {
                self.registers.b = self.registers.b & !(1 << 0);
                2
            },

            0x81 => {
                self.registers.c = self.registers.c & !(1 << 0);
                2
            },

            0x82 => {
                self.registers.d = self.registers.d & !(1 << 0);
                2
            },

            0x83 => {
                self.registers.e = self.registers.e & !(1 << 0);
                2
            },

            0x84 => {
                self.registers.h = self.registers.h & !(1 << 0);
                2
            },

            0x85 => {
                self.registers.l = self.registers.l & !(1 << 0);
                2
            },

            0x86 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 0);
                self.mmu.write_byte(a, v);

                4
            },

            0x87 => {
                self.registers.a = self.registers.a & !(1 << 0);
                2
            },

            0x88 => {
                self.registers.b = self.registers.b & !(1 << 1);
                2
            },

            0x89 => {
                self.registers.c = self.registers.c & !(1 << 1);
                2
            },

            0x8A => {
                self.registers.d = self.registers.d & !(1 << 1);
                2
            },

            0x8B => {
                self.registers.e = self.registers.e & !(1 << 1);
                2
            },

            0x8C => {
                self.registers.h = self.registers.h & !(1 << 1);
                2
            },

            0x8D => {
                self.registers.l = self.registers.l & !(1 << 1);
                2
            },

            0x8E => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 1);
                self.mmu.write_byte(a, v);

                4
            },

            0x8F => {
                self.registers.a = self.registers.a & !(1 << 1);
                2
            },

            0x90 => {
                self.registers.b = self.registers.b & !(1 << 2);
                2
            },

            0x91 => {
                self.registers.c = self.registers.c & !(1 << 2);
                2
            },

            0x92 => {
                self.registers.d = self.registers.d & !(1 << 2);
                2
            },

            0x93 => {
                self.registers.e = self.registers.e & !(1 << 2);
                2
            },

            0x94 => {
                self.registers.h = self.registers.h & !(1 << 2);
                2
            },

            0x95 => {
                self.registers.l = self.registers.l & !(1 << 2);
                2
            },

            0x96 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 2);
                self.mmu.write_byte(a, v);

                4
            },

            0x97 => {
                self.registers.a = self.registers.a & !(1 << 2);
                2
            },

            0x98 => {
                self.registers.b = self.registers.b & !(1 << 3);
                2
            },

            0x99 => {
                self.registers.c = self.registers.c & !(1 << 3);
                2
            },

            0x9A => {
                self.registers.d = self.registers.d & !(1 << 3);
                2
            },

            0x9B => {
                self.registers.e = self.registers.e & !(1 << 3);
                2
            },

            0x9C => {
                self.registers.h = self.registers.h & !(1 << 3);
                2
            },

            0x9D => {
                self.registers.l = self.registers.l & !(1 << 3);
                2
            },

            0x9E => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 3);
                self.mmu.write_byte(a, v);

                4
            },

            0x9F => {
                self.registers.a = self.registers.a & !(1 << 3);
                2
            },

            0xA0 => {
                self.registers.b = self.registers.b & !(1 << 4);
                2
            },

            0xA1 => {
                self.registers.c = self.registers.c & !(1 << 4);
                2
            },

            0xA2 => {
                self.registers.d = self.registers.d & !(1 << 4);
                2
            },

            0xA3 => {
                self.registers.e = self.registers.e & !(1 << 4);
                2
            },

            0xA4 => {
                self.registers.h = self.registers.h & !(1 << 4);
                2
            },

            0xA5 => {
                self.registers.l = self.registers.l & !(1 << 4);
                2
            },

            0xA6 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 4);
                self.mmu.write_byte(a, v);

                4
            },

            0xA7 => {
                self.registers.a = self.registers.a & !(1 << 4);
                2
            },

            0xA8 => {
                self.registers.b = self.registers.b & !(1 << 5);
                2
            },

            0xA9 => {
                self.registers.c = self.registers.c & !(1 << 5);
                2
            },

            0xAA => {
                self.registers.d = self.registers.d & !(1 << 5);
                2
            },

            0xAB => {
                self.registers.e = self.registers.e & !(1 << 5);
                2
            },

            0xAC => {
                self.registers.h = self.registers.h & !(1 << 5);
                2
            },

            0xAD => {
                self.registers.l = self.registers.l & !(1 << 5);
                2
            },

            0xAE => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 5);
                self.mmu.write_byte(a, v);

                4
            },

            0xAF => {
                self.registers.a = self.registers.a & !(1 << 5);
                2
            },

            0xB0 => {
                self.registers.b = self.registers.b & !(1 << 6);
                2
            },

            0xB1 => {
                self.registers.c = self.registers.c & !(1 << 6);
                2
            },

            0xB2 => {
                self.registers.d = self.registers.d & !(1 << 6);
                2
            },

            0xB3 => {
                self.registers.e = self.registers.e & !(1 << 6);
                2
            },

            0xB4 => {
                self.registers.h = self.registers.h & !(1 << 6);
                2
            },

            0xB5 => {
                self.registers.l = self.registers.l & !(1 << 6);
                2
            },

            0xB6 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 6);
                self.mmu.write_byte(a, v);

                4
            },

            0xB7 => {
                self.registers.a = self.registers.a & !(1 << 6);
                2
            },

            0xB8 => {
                self.registers.b = self.registers.b & !(1 << 7);
                2
            },

            0xB9 => {
                self.registers.c = self.registers.c & !(1 << 7);
                2
            },

            0xBA => {
                self.registers.d = self.registers.d & !(1 << 7);
                2
            },

            0xBB => {
                self.registers.e = self.registers.e & !(1 << 7);
                2
            },

            0xBC => {
                self.registers.h = self.registers.h & !(1 << 7);
                2
            },

            0xBD => {
                self.registers.l = self.registers.l & !(1 << 7);
                2
            },

            0xBE => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) & !(1 << 7);
                self.mmu.write_byte(a, v);
                4
            },

            0xBF => {
                self.registers.a = self.registers.a & !(1 << 7);
                2
            },

            0xC0 => {
                self.registers.b = self.registers.b | (1 << 0);
                2
            },

            0xC1 => {
                self.registers.c = self.registers.c | (1 << 0);
                2
            },

            0xC2 => {
                self.registers.d = self.registers.d | (1 << 0);
                2
            },

            0xC3 => {
                self.registers.e = self.registers.e | (1 << 0);
                2
            },

            0xC4 => {
                self.registers.h = self.registers.h | (1 << 0);
                2
            },

            0xC5 => {
                self.registers.l = self.registers.l | (1 << 0);
                2
            },

            0xC6 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 0);
                self.mmu.write_byte(a, v);

                4
            },

            0xC7 => {
                self.registers.a = self.registers.a | (1 << 0);
                2
            },

            0xC8 => {
                self.registers.b = self.registers.b | (1 << 1);
                2
            },

            0xC9 => {
                self.registers.c = self.registers.c | (1 << 1);
                2
            },

            0xCA => {
                self.registers.d = self.registers.d | (1 << 1);
                2
            },

            0xCB => {
                self.registers.e = self.registers.e | (1 << 1);
                2
            },

            0xCC => {
                self.registers.h = self.registers.h | (1 << 1);
                2
            },

            0xCD => {
                self.registers.l = self.registers.l | (1 << 1);
                2
            },

            0xCE => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 1);
                self.mmu.write_byte(a, v);

                4
            },

            0xCF => {
                self.registers.a = self.registers.a | (1 << 1);
                2
            },

            0xD0 => {
                self.registers.b = self.registers.b | (1 << 2);
                2
            },

            0xD1 => {
                self.registers.c = self.registers.c | (1 << 2);
                2
            },

            0xD2 => {
                self.registers.d = self.registers.d | (1 << 2);
                2
            },

            0xD3 => {
                self.registers.e = self.registers.e | (1 << 2);
                2
            },

            0xD4 => {
                self.registers.h = self.registers.h | (1 << 2);
                2
            },

            0xD5 => {
                self.registers.l = self.registers.l | (1 << 2);
                2
            },

            0xD6 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 2);
                self.mmu.write_byte(a, v);

                4
            },

            0xD7 => {
                self.registers.a = self.registers.a | (1 << 2);
                2
            },

            0xD8 => {
                self.registers.b = self.registers.b | (1 << 3);
                2
            },

            0xD9 => {
                self.registers.c = self.registers.c | (1 << 3);
                2
            },

            0xDA => {
                self.registers.d = self.registers.d | (1 << 3);
                2
            },

            0xDB => {
                self.registers.e = self.registers.e | (1 << 3);
                2
            },

            0xDC => {
                self.registers.h = self.registers.h | (1 << 3);
                2
            },

            0xDD => {
                self.registers.l = self.registers.l | (1 << 3);
                2
            },

            0xDE => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 3);
                self.mmu.write_byte(a, v);

                4
            },

            0xDF => {
                self.registers.a = self.registers.a | (1 << 3);
                2
            },

            0xE0 => {
                self.registers.b = self.registers.b | (1 << 4);
                2
            },

            0xE1 => {
                self.registers.c = self.registers.c | (1 << 4);
                2
            },

            0xE2 => {
                self.registers.d = self.registers.d | (1 << 4);
                2
            },

            0xE3 => {
                self.registers.e = self.registers.e | (1 << 4);
                2
            },

            0xE4 => {
                self.registers.h = self.registers.h | (1 << 4);
                2
            },

            0xE5 => {
                self.registers.l = self.registers.l | (1 << 4);
                2
            },

            0xE6 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 4);
                self.mmu.write_byte(a, v);

                4
            },

            0xE7 => {
                self.registers.a = self.registers.a | (1 << 4);
                2
            },

            0xE8 => {
                self.registers.b = self.registers.b | (1 << 5);
                2
            },

            0xE9 => {
                self.registers.c = self.registers.c | (1 << 5);
                2
            },

            0xEA => {
                self.registers.d = self.registers.d | (1 << 5);
                2
            },

            0xEB => {
                self.registers.e = self.registers.e | (1 << 5);
                2
            },

            0xEC => {
                self.registers.h = self.registers.h | (1 << 5);
                2
            },

            0xED => {
                self.registers.l = self.registers.l | (1 << 5);
                2
            },

            0xEE => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 5);
                self.mmu.write_byte(a, v);

                4
            },

            0xEF => {
                self.registers.a = self.registers.a | (1 << 5);
                2
            },

            0xF0 => {
                self.registers.b = self.registers.b | (1 << 6);
                2
            },

            0xF1 => {
                self.registers.c = self.registers.c | (1 << 6);
                2
            },

            0xF2 => {
                self.registers.d = self.registers.d | (1 << 6);
                2
            },

            0xF3 => {
                self.registers.e = self.registers.e | (1 << 6);
                2
            },

            0xF4 => {
                self.registers.h = self.registers.h | (1 << 6);
                2
            },

            0xF5 => {
                self.registers.l = self.registers.l | (1 << 6);
                2
            },

            0xF6 => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 6);
                self.mmu.write_byte(a, v);

                4
            },

            0xF7 => {
                self.registers.a = self.registers.a | (1 << 6);
                2
            },

            0xF8 => {
                self.registers.b = self.registers.b | (1 << 7);
                2
            },

            0xF9 => {
                self.registers.c = self.registers.c | (1 << 7);
                2
            },

            0xFA => {
                self.registers.d = self.registers.d | (1 << 7);
                2
            },

            0xFB => {
                self.registers.e = self.registers.e | (1 << 7);
                2
            },

            0xFC => {
                self.registers.h = self.registers.h | (1 << 7);
                2
            },

            0xFD => {
                self.registers.l = self.registers.l | (1 << 7);
                2
            },

            0xFE => {
                let a = self.registers.hl();
                let v = self.mmu.read_byte(a) | (1 << 7);
                self.mmu.write_byte(a, v);

                4
            },

            0xFF => {
                self.registers.a = self.registers.a | (1 << 7);
                2
            }
        }
    }

    /// Performs an addition
    fn alu_add(&mut self, b: u8, usec: bool) {
        let c = if usec && self.registers.is_flag_set(C) { 1 } else { 0 };
        let a = self.registers.a;

        let r = a.wrapping_add(b).wrapping_add(c);

        self.registers.flag(Z, r == 0);
        self.registers.flag(H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.registers.flag(N, false);
        self.registers.flag(C, (a as u16) + (b as u16) + (c as u16) > 0xFF);

        self.registers.a = r;
    }

    /// Performs a subtraction
    ///
    /// More details in the code
    fn alu_subtract(&mut self, b: u8, usec: bool) {
        let c = if usec && self.registers.is_flag_set(C) { 1 } else { 0 };
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

    fn alu_compare(&mut self, b: u8) {
        let r = self.registers.a;
        self.alu_subtract(b, false);

        self.registers.a = r;
    }

    fn alu_increase(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.registers.flag(Z, r == 0);
        self.registers.flag(H, (a & 0x0F) + 1 > 0x0F);
        self.registers.flag(N, false);

        return r
    }

    fn alu_decrease(&mut self, a: u8) -> u8 {
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
        let b = self.read_byte() as i8 as i16 as u16;
        self.registers.flag(N, false);
        self.registers.flag(Z, false);
        self.registers.flag(H, (a & 0x000F) + (b & 0x000F) > 0x000F);
        self.registers.flag(C, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);

        return a.wrapping_add(b)
    }

    /// Status Register (SR) flag update
    fn alu_sr_flagupdate(&mut self, r: u8, carry: bool) {
        self.registers.flag(H, false);
        self.registers.flag(N, false);
        self.registers.flag(Z, r == 0);
        self.registers.flag(C, carry);
    }

    /// Rotate Left with Carry (RLC) operation
    ///
    /// 8-bit rotation to the left. The bit leaving on the left
    /// is copied into the carry, and to bit 0.
    fn alu_rlc(&mut self, a: u8) -> u8 {
        let carry = a & 0x80 == 0x80;

        let r = (a << 1) | (if carry { 1 } else { 0 });

        self.alu_sr_flagupdate(r, carry);

        return r
    }

    /// Rotate Left (RL) operation
    fn alu_rl(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.registers.is_flag_set(C) { 1 } else { 0 });
        self.alu_sr_flagupdate(r, c);

        return r
    }

    /// Rotate Right Circula (RRC) operation
    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0 });
        self.alu_sr_flagupdate(r, c);

        return r
    }

    /// Rotate Right (RR) operation
    fn alu_rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if self.registers.is_flag_set(C) { 0x80 } else { 0 });
        self.alu_sr_flagupdate(r, c);

        return r
    }

    /// Shift-Left Arithmetic (SLA) operation
    fn alu_sla(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = a << 1;
        self.alu_sr_flagupdate(r, c);

        return r
    }

    /// Shift-Right Arithmetic (SRA) operation
    fn alu_sra(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);
        self.alu_sr_flagupdate(r, c);

        return r
    }

    /// Shift-Right Logical
    fn alu_srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;
        self.alu_sr_flagupdate(r, c);

        return r
    }

    /// Bit test operation
    fn alu_bit(&mut self, a: u8, b: u8) {
        let r = a & (1 << (b as u32)) == 0;

        self.registers.flag(N, false);
        self.registers.flag(H, true);
        self.registers.flag(Z, r);
    }

    fn alu_swap(&mut self, a: u8) -> u8 {
        self.registers.flag(Z, a == 0);
        self.registers.flag(C, false);
        self.registers.flag(H, false);
        self.registers.flag(N, false);

        (a >> 4) | (a << 4)
    }

    /// Decimal Adjust Accumulator (DAA)
    fn alu_daa(&mut self) {
        let mut a = self.registers.a;
        let mut adjust = if self.registers.is_flag_set(C) { 0x60 } else { 0x00 };

        if self.registers.is_flag_set(H) { adjust |= 0x06; };

        if !self.registers.is_flag_set(N) {
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

    /// Jump Relative (JR) CPU functionality
    fn cpu_jr(&mut self) {
        let n = self.read_byte() as i8;
        self.registers.program_counter = ((self.registers.program_counter as u32 as i32) + (n as i32)) as u16;
    }

    pub fn get_gpu_pixels(&self) -> &[u8] {
        &self.mmu.gpu.raw_pixels
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