use memory::mbc;
use cpu::timer::Timer;
use frontend::keypad::Keypad;
use gpu::gpu::GPU;

/// Working RAM, 8k bytes
const WORKING_RAM_SIZE: usize = 0x8000;

/// High RAM (Zero Page), 127 bytes
const HIGH_RAM_SIZE: usize = 0x7F;


/// Memory Management Unit (MMU)
///
/// This is central part of the GameBoy where all the memory
/// is stored (including the VRAM, abstracted as a GPU here, but
/// it's actually inside the MMU chip).
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

    /// Interrupt enable (IE) register
    ///
    /// The interrupt stored here is the one that the CPU will
    /// handle in case the IME is enabled (set to 1).
    ///
    /// See the CPU `interrupt_master_enable` documentation for more info
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
        self.write_byte(0xFF05, 0);

        // Timer modulo (TMA)
        self.write_byte(0xFF06, 0);

        // Timer control (TAC)
        self.write_byte(0xFF07, 0);

        // LCD control (LCDC)
        self.write_byte(0xFF40, 0x91);

        // LCD position (Scroll Y) (SCY)
        self.write_byte(0xFF42, 0);

        // LCD position (Scroll X) (SCX)
        self.write_byte(0xFF43, 0);

        // LCD Y-coordinate (LY)
        self.write_byte(0xFF44, 0);

        // LCD LY Compare (LYC)
        self.write_byte(0xFF45, 0);

        // LCD Window Y Position
        self.write_byte(0xFF4A, 0);

        // LCD Window X Position
        self.write_byte(0xFF4B, 0);

        // BG Palette (BGP)
        self.write_byte(0xFF47, 0xFC);

        // Spritte Palette 0
        self.write_byte(0xFF48, 0xFF);

        // Spritte Palette 1
        self.write_byte(0xFF49, 0xFF);
    }

    /// Steps the MMU
    ///
    /// This will handle interrupts from implemented sources
    /// (timer, GPU and keypad) and also cycle the GPU and the Timer
    pub fn step(&mut self, ticks: u32) {
        // cycle the timer and check for interrupts
        self.timer.step(ticks);
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
    pub fn read_byte(&mut self, address: u16) -> u8 {
        match address {

            0x0000 ... 0x7FFF => {
                self.mbc.read_rom(address)
            },

            0x8000 ... 0x9FFF => {
                self.gpu.read_byte(address)
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
                self.gpu.read_byte(address)
            },

            0xFF00 => {
                self.keypad.read_byte()
            },

            0xFF01 ... 0xFF02 => {
                // Serial unimplemented
                0x0
            },

            0xFF04 ... 0xFF07 => {
                self.timer.read_byte(address)
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
                self.gpu.read_byte(address)
            },

            0xFF68 ... 0xFF6B => {
                self.gpu.read_byte(address)
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

    pub fn read_word(&mut self, address: u16) -> u16 {
        (self.read_byte(address) as u16) |
            ((self.read_byte(address + 1) as u16) << 8)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // extra MBC memory. see more details in the MBC module
            0x0000 ... 0x7FFF =>  {
                self.mbc.write_rom(address, value)
            },

            0x8000 ... 0x9FFF => {
                self.gpu.write_byte(address, value)
            },

            // more MBC memory!
            0xA000 ... 0xBFFF => {
                self.mbc.write_ram(address, value)
            },

            // internal working ram (bank 0)
            0xC000 ... 0xCFFF | 0xE000 ... 0xEFFF => {
                self.working_ram[address as usize & 0x0FFF] = value
            },

            // internal working ram (bank 1)
            0xD000 ... 0xDFFF | 0xF000 ... 0xFDFF => {
                self.working_ram[0x1000 | (address as usize & 0x0FFF)] = value
            },

            // gpu, mapped to OAM (object attribute memory)
            0xFE00 ... 0xFE9F => {
                self.gpu.write_byte(address, value)
            },

            // first of I/O ports, the keypad
            0xFF00 => {
                self.keypad.write_byte(value)
            },

            // serial port, not implemented
            0xFF01 ... 0xFF03 => {
            }

            // timer
            0xFF04 ... 0xFF07 => {
                self.timer.write_byte(address, value)
            },

            // sound, unimplemented
            0xFF10 ... 0xFF3F => {
            },

            // DMA (Direct Memory Access) transfer from
            // RAM to OAM
            0xFF46 => {
                self.dma_ram_to_oam_transfer(value)
            },

            0xFF40 ... 0xFF4F => {
                self.gpu.write_byte(address, value)
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

            unimplemented => println!("Unimplemented memory address: {0:x}", unimplemented),
        };
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    fn dma_ram_to_oam_transfer(&mut self, value: u8) {
        let base = (value as u16) << 8;

        for i in 0 .. 0xA0 {
            let b = self.read_byte(base + i);
            self.write_byte(0xFE00 + i, b);
        }
    }
}
