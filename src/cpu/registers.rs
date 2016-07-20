/// The GameBoy Z80 CPU Registers
///
/// These registers are used by the CPU to perform calculation. They
/// are operated through opcodes (available at the CPU).
///
/// The GameBoy Z80 has 7 8-bit registers (values from 0 to 255)
/// and 2 16-bit registers (values from 0 to 65535)
///
/// It also contains flags register, used to stored results from
/// operations (Zero, Operation, Half-carry and Carry)
#[derive(Copy, Clone)]
pub struct RegisterSet {
    // 8-bit registers (nameless)
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    /// Flags register
    ///
    /// Used to store results from operations:
    ///
    /// Bit 0 -> If the result from last operation was 0
    /// Bit 1 -> If the operation was a subtraction
    /// Bit 2 -> Set if the lower half of the byte overflowed past 15
    /// Bit 3 -> Set if last operation produced an overflow over 255 or under 0
    pub flags: u8,

    /// Program Counter
    ///
    /// Stores the position in which the CPU is when executing the
    /// program. It's advanced after an instruction is fetched from
    /// memory. It's in the register
    pub program_counter: u16,

    /// Stack pointer
    ///
    /// This is a basic stack, used to perform LIFO data storage.
    /// This will to ease usage of functions calls, interrupts and
    /// temporar data storage
    pub stack_pointer: u16,
}

/// Flags
///
/// These are each of the raw flag values
/// used to set the flags register.
#[derive(Copy, Clone)]
pub enum CpuFlag
{
    C = 0b00010000, // Carry
    H = 0b00100000, // Half carry
    N = 0b01000000, // Subtraction
    Z = 0b10000000, // Zero
}

impl RegisterSet {
    pub fn new() -> RegisterSet {
        // these register initial values are taken
        // from original hardware (as state left by the
        // GameBoy boot ROM)
        RegisterSet {
            a: 0b0000_0001,
            b: 0b0000_0000,
            c: 0b0001_0011,
            d: 0x0000_0000,
            e: 0b1101_1000,
            h: 0b0000_0001,
            l: 0b0100_1101,

            flags: 0b1011_0000,

            program_counter: 0b0000_0001_0000_0000,
            stack_pointer: 0b1111_1111_1111_1110,
        }
    }

    /// Grouped AF register
    ///
    /// Returns the A and F registers grouped as a 16-bit register
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) |
            ((self.flags & 0xF0) as u16)
    }

    /// Grouped BC register
    ///
    /// Returns the B and C registers grouped as a 16-bit register
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) |
            (self.c as u16)
    }

    /// Grouped DE register
    ///
    /// Returns the D and E registers grouped as a 16-bit register
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) |
            (self.e as u16)
    }

    /// Grouped HL register
    ///
    /// Returns the H and L registers grouped as a 16-bit register
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) |
            (self.l as u16)
    }

    /// HL decrease
    ///
    /// Gets the value from the HL grouped register
    /// and decrease minus 1
    pub fn hl_decrease(&mut self) -> u16 {
        let res = self.hl();
        self.set_hl(res - 1);
        res
    }

    /// HL increase
    ///
    /// Gets the value from the HL grouped register
    /// and increase plus 1
    pub fn hl_increase(&mut self) -> u16 {
        let res = self.hl();
        self.set_hl(res + 1);
        
        res
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.flags = (value & 0x00F0) as u8;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    /// Set register flags
    ///
    /// Set (or unset) flags by masking bits
    pub fn flag(&mut self, flags: CpuFlag, set: bool) {
        let mask = flags as u8;

        match set {
            true  => self.flags |=  mask,
            false => self.flags &= !mask,
        }

        self.flags &= 0xF0;
    }

    /// Returns whether a flag is set or not
    pub fn is_flag_set(&self, flags: CpuFlag) -> bool {
        let mask = flags as u8; // convert enum to u8 values
        self.flags & mask > 0 // mask and compare
    }
}