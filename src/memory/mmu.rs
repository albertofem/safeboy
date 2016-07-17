use memory::mbc;
use cpu::timer::Timer;
use frontend::keypad::Keypad;
use gpu::gpu::GPU;

/// Working RAM, 8k bytes
const WORKING_RAM_SIZE: usize = 0x8000;

/// High RAM (Zero Page), 127 bytes
const HIGH_RAM_SIZE: usize = 0x7F;

pub struct MMU {
    /// Working RAM, with 8K Byte of size
    ///
    /// * It's the internal GameBoy ram, shipped with the console
    /// * It has two main banks accessibles in addresses 0xC000-0xCFFF and 0xD000-0xDFFF
    /// * These addresses are echoed in the range 0xE000-0xFDFF, but it's tipically not used
    working_ram: [u8; WORKING_RAM_SIZE],

    /// High RAM, also called Zero-page RAM, with 127 bytes of size
    ///
    /// * This is a super-speed RAM space use for the program stack
    /// * It's memory mapped I/O at address space 0xFF00-0xFFFE
    high_ram: [u8; HIGH_RAM_SIZE],

    /// Interrupt master Enable (IME) register
    ///
    /// * This is a special register used to enable/disable interrupts
    /// * If this bit is set, then no interrupts are handled by the CPU
    pub interrupt_enable: u8,

    /// Interrupt request register Flag (IF)
    ///
    /// * This is where interrupts are requested to the CPU
    /// * It contains a bit indicating what kind of interrupt ocurre
    ///
    /// ** 0 V-Blank (GPU)
    /// ** 1 LCD STAT (GPU)
    /// ** 2 Timer
    /// ** 3 Serial port
    /// ** 4 Keypad
    pub interrupt_flag: u8,

    /// Internal timer
    ///
    /// The GameBoy has an internal timer providing games
    /// the possibility of basing behaviour on a rate.
    ///
    /// More details in the module.
    pub timer: Timer,

    /// Keypad
    ///
    /// This is the GameBoy gamepad, used by the user
    /// to interact with the game. More details in the module
    pub keypad: Keypad,

    /// Graphic Processing Unit
    ///
    /// This is where the screen pixeles are calculated
    /// based on the original tiles and sprites stored in the
    /// memory. More details in the module.
    pub gpu: GPU,

    /// Memory Bank Controller
    ///
    /// The memory bank controller is a chip inside cartdriges
    /// that allowed games to provide more content with the limited
    /// memory constrains of the GameBoy
    ///
    /// More details in the module.
    pub mbc: Box<mbc::MBC+'static>,
}

impl MMU {
    pub fn new(rom_file: &str) -> MMU {
        // load the file raw data into the MBC, where the ERAM is located
        let mbc = mbc::load_mbc(rom_file).unwrap();

        let mut mmu = MMU {
            working_ram: [0; WORKING_RAM_SIZE],
            high_ram: [0; HIGH_RAM_SIZE],

            interrupt_enable: 0,
            interrupt_flag: 0,

            timer: Timer::new(),
            keypad: Keypad::new(),
            gpu: GPU::new(),
            mbc: mbc
        };

        mmu.reset();

        mmu
    }

    fn reset(&mut self) {
        // Timer counter (TIMA)
        self.wb(0xFF05, 0);

        // Timer modulo (TMA)
        self.wb(0xFF06, 0);

        // Timer control (TAC)
        self.wb(0xFF07, 0);

        // LCD control (LCDC)
        self.wb(0xFF40, 0x91);

        // LCD position (Scroll Y) (SCY)
        self.wb(0xFF42, 0);

        // LCD position (Scroll X) (SCX)
        self.wb(0xFF43, 0);

        // LCD Y-coordinate (LY)
        self.wb(0xFF44, 0);

        // LCD LY Compare (LYC)
        self.wb(0xFF45, 0);

        // LCD Window Y Position
        self.wb(0xFF4A, 0);

        // LCD Window X Position
        self.wb(0xFF4B, 0);

        // BG Palette (BGP)
        self.wb(0xFF47, 0xFC);

        // Spritte Palette 0
        self.wb(0xFF48, 0xFF);

        // Spritte Palette 1
        self.wb(0xFF49, 0xFF);
    }

    /// Steps the MMU
    ///
    /// This will handle interrupts from implemented sources
    /// (timer, GPU and keypad) and also cycle the GPU and the Timer
    pub fn step(&mut self, ticks: u32) {
        // cycle the timer and check for interrupts
        self.timer.do_cycle(ticks);
        self.interrupt_flag |= self.timer.interrupt;

        // check for keypad interrupts
        // keypad is not cycled because interrupt data
        // is directly handled by user input
        self.interrupt_flag |= self.keypad.interrupt;

        // cycle the GPU and check for GPU interrupts
        self.gpu.step(ticks);
        self.interrupt_flag |= self.gpu.interrupt;

        // reset interrupts
        self.keypad.interrupt = 0;
        self.timer.interrupt = 0;
        self.gpu.interrupt = 0;
    }

    /// Read a byte from the MMU
    ///
    /// Providing a valid address, the MMU will return the
    /// value found in the address space. Some addresses are mapped
    /// to GPU, timer, keypad, etc. addresses, but this is handled
    /// internally
    pub fn rb(&mut self, address: u16) -> u8 {
        match address {

            0x0000 ... 0x7FFF => {
                self.mbc.read_rom(address)
            },

            0x8000 ... 0x9FFF => {
                self.gpu.rb(address)
            },

            0xA000 ... 0xBFFF => {
                self.mbc.read_ram(address)
            },

            0xC000 ... 0xCFFF | 0xE000 ... 0xEFFF => {
                self.working_ram[address as usize & 0x0FFF]
            },

            0xD000 ... 0xDFFF | 0xF000 ... 0xFDFF => {
                self.working_ram[0x1000 | address as usize & 0x0FFF]
            },

            0xFE00 ... 0xFE9F => {
                self.gpu.rb(address)
            },

            0xFF00 => {
                self.keypad.rb()
            },

            0xFF01 ... 0xFF02 => {
                // Serial unimplemented
                0x0
            },

            0xFF04 ... 0xFF07 => {
                self.timer.rb(address)
            },

            0xFF0F => {
                self.interrupt_flag
            },

            0xFF10 ... 0xFF3F => {
                // Sound unimplemented
                0x0
            },

            0xFF4D => {
                0
            },

            0xFF40 ... 0xFF4F => {
                self.gpu.rb(address)
            },

            0xFF68 ... 0xFF6B => {
                self.gpu.rb(address)
            },

            0xFF80 ... 0xFFFE => {
                self.high_ram[address as usize & 0x007F]
            },

            0xFFFF => {
                self.interrupt_enable
            },

            _ => 0,
        }
    }

    pub fn rw(&mut self, address: u16) -> u16 {
        (self.rb(address) as u16) |
            ((self.rb(address + 1) as u16) << 8)
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x7FFF =>  {
                self.mbc.write_rom(address, value)
            },

            0x8000 ... 0x9FFF => {
                self.gpu.wb(address, value)
            },

            0xA000 ... 0xBFFF => {
                self.mbc.write_ram(address, value)
            },

            0xC000 ... 0xCFFF | 0xE000 ... 0xEFFF => {
                self.working_ram[address as usize & 0x0FFF] = value
            },

            0xD000 ... 0xDFFF | 0xF000 ... 0xFDFF => {
                self.working_ram[0x1000 | (address as usize & 0x0FFF)] = value
            },

            0xFE00 ... 0xFE9F => {
                self.gpu.wb(address, value)
            },

            0xFF00 => {
                self.keypad.wb(value)
            },

            0xFF04 ... 0xFF07 => {
                self.timer.wb(address, value)
            },

            0xFF10 ... 0xFF3F => {
                // Sound unimplemented
            },

            0xFF46 => {
                self.oamdma(value)
            },

            0xFF40 ... 0xFF4F => {
                self.gpu.wb(address, value)
            },

            0xFF68 ... 0xFF6B => {
                self.gpu.wb(address, value)
            },

            0xFF0F => {
                self.interrupt_flag = value
            },

            0xFF80 ... 0xFFFE => {
                self.high_ram[address as usize & 0x007F] = value
            },

            0xFFFF => {
                self.interrupt_enable = value
            },

            0xFF01 ... 0xFF02 => {
                // Serial port unimplemented
            },

            unimplemented => println!("Unimplemented memory instruction: {0:x}", unimplemented),
        };
    }

    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8);
        self.wb(address + 1, (value >> 8) as u8);
    }

    fn oamdma(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for i in 0 .. 0xA0 {
            let b = self.rb(base + i);
            self.wb(0xFE00 + i, b);
        }
    }
}
