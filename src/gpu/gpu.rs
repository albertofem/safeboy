
/// Video Ram size,
const VRAM_SIZE: usize = 0x8000;
const VOAM_SIZE: usize = 0xA0;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

#[derive(PartialEq, Copy, Clone)]
enum PrioType {
    Color0,
    Normal,
}

#[derive(PartialEq, Copy, Clone)]
enum Mode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    OAMRead = 2,
    VRAMRead = 3
}

struct Control {
    lcd_display_enable: bool,
    window_tile_map_display_base_address_base_address: u16,
    window_display_enable: bool,
    bg_window_tile_data_base_address: u16,
    background_tile_data_base_address: u16
}

/// Graphic Processing Unit
///
/// This is where the all the graphics processing happens.
/// It's also called the LCD.
///
/// The GPU contains all graphic data, which is
pub struct GPU {
    /// Mode flag
    ///
    /// This is where we set in which mode the GPU is, like
    /// a machine state, for other parts to know (like the CPU)
    mode: Mode,

    /// LCD display enable flag
    ///
    /// This is a bit indicating whether the display is enabled
    /// or not, used by the games to disable it. During the disabled
    /// period, the screen is blank and VRAM and OAM can be accessed freely
    lcd_display_enable: bool,

    /// Window tilemap display base address
    ///
    /// This bit indicates the initial position where
    /// tilemap can be read.
    ///
    /// 0 -> 9800-9BFF, 1 -> 9C00-9FFF
    ///
    /// It's represented as a u16, because we store the address directly
    /// instead of the bit
    window_tile_map_display_base_address: u16,

    /// Window display enable
    ///
    /// This bit indicates whether the display is enabled or not
    /// When disable, the GPU won't draw the window
    window_display_enable: bool,

    /// Background & Window tile data base address
    ///
    /// This indicates where is the base address at which
    /// we can read bg and window tile data.
    ///
    /// 0 -> 8800-97FF, 1 -> 8000-8FFF
    bg_window_tile_data_base_address: u16,

    /// Background tile map base address
    ///
    /// Same as before, but for the background tilemap data
    ///
    /// 0 -> 9800-9BFF, 1 ->9C00-9FFF
    bg_tile_map_base_address: u16,

    /// Sprite size
    ///
    /// This store the sprite size that we need to draw. The GameBoy
    /// supported two sprite sizes: 8x8 and 8x16
    sprite_size: u32,

    /// Sprite enable
    /// 
    /// A flag to indicate whether the GPU should draw the sprites
    /// or not on the screen
    sprite_enable: bool,

    /// Background enable
    ///
    /// A flag to indicate if we need to draw the background
    /// or not on the screen
    background_display_enable: bool,

    /// GPU clock
    ///
    /// This internal clocks, it's used to jump from one mode to another
    /// when rendering in the GPU. Each mode has a specific timing that 
    /// is used to determine whether we need to jump to another mode. These
    /// timings are controlled in this variable
    clock: u32,

    /// Line
    ///
    /// This is the current line beign rendered by the GPU. It's used
    /// to control how much lines a given GPU mode should draw and also
    /// to calculate the LYC value (see below).
    line: u8,

    /// LCD Y Coordinate comparison
    ///
    /// This register is used to store a value that will be compared with
    /// the current Y Coordinate line beign rendered. If the comparison
    /// is true, an interrupt is requested.
    lyc: u8,
    
    /// LCD Y Coordinate interrupt
    ///
    /// This interrupt will happen when the current line beign rendered
    /// is the same as the Y coordinate.
    ///
    /// Notice that this isn't the actual interrupt, but an internal flag.
    /// The actual interrupt occurs in the "interrupt", which is read (see below)
    /// by the MMU when stepping.
    lyc_interrupt: bool,
    
    /// Horizontal-Vertical blank and OAM interrupt
    ///
    /// Whether the GPU is on each of these mode. These flags
    /// are returned as a result of a STAT operation.
    horizontal_blank_interrupt: bool,
    vertical_blank_interrupt: bool,
    oam_interrupt: bool,

    /// Scroll Position Y
    ///
    /// Stores the scroll Y-coordinate position. The value is ranged 0-255
    scroll_position_y: u8,

    /// Scroll Position X
    ///
    /// Stores the scroll X-coordinate position. The value is ranged 0-255
    scroll_position_x: u8,

    /// Window position Y
    ///
    /// Stores the window Y-coordinate position.
    window_position_y: u8,

    /// Window position X
    ///
    /// Stores the window X-coordinate position.
    window_position_x: u8,

    palbr: u8,
    pal0r: u8,
    pal1r: u8,

    palb: [u8; 4],
    pal0: [u8; 4],
    pal1: [u8; 4],

    vram: [u8; VRAM_SIZE],
    voam: [u8; VOAM_SIZE],

    bgprio: [PrioType; WIDTH],

    /// GPU Interrupt
    ///
    /// The GPU has 2 interrupts:
    ///
    /// * When the V-Blank period starts, during this period the VRAM is accessible
    /// *
    pub interrupt: u8,
    pub data: Vec<u8>
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            mode: Mode::HorizontalBlank,
            clock: 0,
            line: 0,
            lyc: 0,
            lcd_display_enable: false,
            window_tile_map_display_base_address: 0x9C00,
            window_display_enable: false,
            bg_window_tile_data_base_address: 0x8000,
            bg_tile_map_base_address: 0x9C00,
            sprite_size: 8,
            sprite_enable: false,
            background_display_enable: false,
            lyc_interrupt: false,
            oam_interrupt: false,
            vertical_blank_interrupt: false,
            horizontal_blank_interrupt: false,
            scroll_position_y: 0,
            scroll_position_x: 0,
            window_position_y: 0,
            window_position_x: 0,
            palbr: 0,
            pal0r: 0,
            pal1r: 1,
            palb: [0; 4],
            pal0: [0; 4],
            pal1: [0; 4],
            vram: [0; VRAM_SIZE],
            voam: [0; VOAM_SIZE],
            data: vec![0; WIDTH * HEIGHT * 3],
            bgprio: [PrioType::Normal; WIDTH],
            interrupt: 0,
        }
    }

    /// Steps the GPU
    /// 
    /// This function is responsible for properly calculating the
    /// screen each CPU step. In order to do that we need to jump
    /// from one mode to another while calculating lines pixels.
    /// 
    /// Notice that we are just calculating pixels at this step, not
    /// rendering them into the screen (this is done by the actual hardware
    /// or in our case, by the OpenGL display module: see there for more details)
    /// 
    /// Details about each one of the jumps are commented in the code.
    pub fn step(&mut self, mut ticks: u32) {
        // if the screen is off, we don't calculate anything
        if !self.lcd_display_enable {
            return
        }

        // while we have ticks left (coming from the CPU)
        // we should keep calculating pixels
        while ticks > 0 {
            // reset the GPU ticks to start at 80
            let gpu_ticks = if ticks >= 80 {
                80
            } else {
                ticks
            };

            self.clock += gpu_ticks;
            ticks -= gpu_ticks;

            if self.clock >= 456 {
                self.clock -= 456;
                self.line = (self.line + 1) % 154;

                self.check_interrupt_lyc();

                if self.line >= 144 && self.mode != Mode::VerticalBlank {
                    self.change_mode(Mode::VerticalBlank);
                }
            }

            if self.line < 144 {
                if self.clock <= 80 {
                    if self.mode != Mode::OAMRead {
                        self.change_mode(Mode::OAMRead);
                    }
                } else if self.clock <= (80 + 172) { // 252 cycles
                    if self.mode != Mode::VRAMRead {
                        self.change_mode(Mode::VRAMRead);
                    }
                } else { // the remaining 204
                    if self.mode != Mode::HorizontalBlank {
                        self.change_mode(Mode::HorizontalBlank);
                    }
                }
            }
        }
    }

    fn check_interrupt_lyc(&mut self) {
        if self.lyc_interrupt && self.line == self.lyc {
            self.interrupt |= 0x02;
        }
    }

    fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;

        if match self.mode {
            Mode::HorizontalBlank => {
                self.renderscan();
                self.horizontal_blank_interrupt
            },
            Mode::VerticalBlank => {
                self.interrupt |= 0x01;
                self.vertical_blank_interrupt
            },
            Mode::OAMRead => self.oam_interrupt,
            _ => false,
        } {
            self.interrupt |= 0x02;
        }
    }

    pub fn rb(&self, a: u16) -> u8 {
        match a {
            0x8000 ... 0x9FFF => self.vram[(a as usize & 0x1FFF)],
            0xFE00 ... 0xFE9F => self.voam[a as usize - 0xFE00],

            0xFF40 => {
                (if self.lcd_display_enable { 0x80 } else { 0 }) |
                    (if self.window_tile_map_display_base_address == 0x9C00 { 0x40 } else { 0 }) |
                    (if self.window_display_enable { 0x20 } else { 0 }) |
                    (if self.bg_window_tile_data_base_address == 0x8000 { 0x10 } else { 0 }) |
                    (if self.bg_tile_map_base_address == 0x9C00 { 0x08 } else { 0 }) |
                    (if self.sprite_size == 16 { 0x04 } else { 0 }) |
                    (if self.sprite_enable { 0x02 } else { 0 }) |
                    (if self.background_display_enable { 0x01 } else { 0 })
            },

            0xFF41 => {
                (if self.lyc_interrupt { 0x40 } else { 0 }) |
                    (if self.oam_interrupt { 0x20 } else { 0 }) |
                    (if self.vertical_blank_interrupt { 0x10 } else { 0 }) |
                    (if self.horizontal_blank_interrupt { 0x08 } else { 0 }) |
                    (if self.line == self.lyc { 0x04 } else { 0 }) |
                    self.mode as u8
            },

            0xFF42 => self.scroll_position_y,
            0xFF43 => self.scroll_position_x,
            0xFF44 => self.line,
            0xFF45 => self.lyc,
            0xFF46 => 0, // Write only
            0xFF47 => self.palbr,
            0xFF48 => self.pal0r,
            0xFF49 => self.pal1r,
            0xFF4A => self.window_position_y,
            0xFF4B => self.window_position_x,
            _ => {
                println!("GPU does not handle read {:04X}", a); 0
            },
        }
    }

    fn rbvram0(&self, a: u16) -> u8 {
        if a < 0x8000 || a >= 0xA000 {
            panic!("Shouldn't have used rbvram0");
        }

        self.vram[a as usize & 0x1FFF]
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x8000 ... 0x9FFF => self.vram[address as usize & 0x1FFF] = value,

            0xFE00 ... 0xFE9F => self.voam[address as usize - 0xFE00] = value,

            0xFF40 => {
                self.handle_gpu_control(value)
            },

            0xFF41 => {
                self.lyc_interrupt = value & 0x40 == 0x40;
                self.oam_interrupt = value & 0x20 == 0x20;
                self.vertical_blank_interrupt = value & 0x10 == 0x10;
                self.horizontal_blank_interrupt = value & 0x08 == 0x08;
            },

            0xFF42 => self.scroll_position_y = value,
            0xFF43 => self.scroll_position_x = value,
            0xFF44 => {}, // Read-only
            0xFF45 => self.lyc = value,
            0xFF46 => panic!("0xFF46 should be handled by MMU"),

            0xFF47 => {
                self.palbr = value; self.update_pal();
            },

            0xFF48 => {
                self.pal0r = value; self.update_pal();
            },

            0xFF49 => {
                self.pal1r = value;
                self.update_pal();
            },

            0xFF4A => self.window_position_y = value,
            0xFF4B => self.window_position_x = value,

            _ => {
                println!("GPU does not handle write {:04X}", address)
            },
        }
    }

    fn handle_gpu_control(&mut self, value: u8) {
        let orig_lcd_display_enable = self.lcd_display_enable;

        // Set lcd display enable bit (convert to boolean)
        self.lcd_display_enable = value & 0x80 == 0x80;

        // Set base address for the window tile map display
        self.window_tile_map_display_base_address = if value & 0x40 == 0x40 {
            0x9C00
        } else {
            0x9800
        };

        // Set window display enable bit (convert to boolean)
        self.window_display_enable = value & 0x20 == 0x20;

        // Set tile map base address
        self.bg_window_tile_data_base_address = if value & 0x10 == 0x10 {
            0x8000
        } else {
            0x8800
        };

        // Set background tile map base address
        self.bg_tile_map_base_address = if value & 0x08 == 0x08 {
            0x9C00
        } else {
            0x9800
        };

        // Set sprite size
        self.sprite_size = if value & 0x04 == 0x04 {
            16
        } else {
            8
        };

        // Set sprite on (convert to boolean)
        self.sprite_enable = value & 0x02 == 0x02;

        // Set background show (convert to boolean)
        self.background_display_enable = value & 0x01 == 0x01;

        // If display was showed but was disable by this operation
        // we need to reset the GPU to initial state
        if orig_lcd_display_enable && !self.lcd_display_enable {
            self.reset();
        }
    }

    /// Reset the GPU
    ///
    /// We first need to reset the internal clock and lines,
    /// also put the mode in Horizontal Blank and clear
    /// the screen with blank pixels
    fn reset(&mut self) {
        self.clock = 0;
        self.line = 0;
        self.mode = Mode::HorizontalBlank;

        for v in self.data.iter_mut() {
            *v = 255;
        }
    }

    fn update_pal(&mut self) {
        for i in 0 .. 4 {
            self.palb[i] = GPU::get_monochrome_pal_val(self.palbr, i);
            self.pal0[i] = GPU::get_monochrome_pal_val(self.pal0r, i);
            self.pal1[i] = GPU::get_monochrome_pal_val(self.pal1r, i);
        }
    }

    fn get_monochrome_pal_val(value: u8, index: usize) -> u8 {
        match (value >> 2*index) & 0x03 {
            0 => 255,
            1 => 192,
            2 => 96,
            _ => 0
        }
    }

    fn renderscan(&mut self) {
        for x in 0 .. WIDTH {
            self.setcolor(x, 255);
            self.bgprio[x] = PrioType::Normal;
        }

        self.draw_bg();
        self.draw_sprites();
    }

    fn setcolor(&mut self, x: usize, color: u8) {
        self.data[self.line as usize * WIDTH * 3 + x * 3 + 0] = color;
        self.data[self.line as usize * WIDTH * 3 + x * 3 + 1] = color;
        self.data[self.line as usize * WIDTH * 3 + x * 3 + 2] = color;
    }

    fn draw_bg(&mut self) {
        let drawbg = self.background_display_enable;

        let window_position_y =
        if !self.window_display_enable || !self.background_display_enable {
            -1
        } else {
            self.line as i32 - self.window_position_y as i32
        };

        if window_position_y < 0 && drawbg == false {
            return;
        }

        let wintiley = (window_position_y as u16 >> 3) & 31;

        let bgy = self.scroll_position_y.wrapping_add(self.line);
        let bgtiley = (bgy as u16 >> 3) & 31;

        for x in 0 .. WIDTH {
            let window_position_x = - ((self.window_position_x as i32) - 7) + (x as i32);
            let bgx = self.scroll_position_x as u32 + x as u32;

            let (tilemapbase, tiley, tilex, pixely, pixelx) = if window_position_y >= 0 && window_position_x >= 0 {
                (self.window_tile_map_display_base_address,
                 wintiley,
                 (window_position_x as u16 >> 3),
                 window_position_y as u16 & 0x07,
                 window_position_x as u8 & 0x07)
            } else if drawbg {
                (self.bg_tile_map_base_address,
                 bgtiley,
                 (bgx as u16 >> 3) & 31,
                 bgy as u16 & 0x07,
                 bgx as u8 & 0x07)
            } else {
                continue;
            };

            let tilenr: u8 = self.rbvram0(tilemapbase + tiley * 32 + tilex);

            let tileaddress = self.bg_window_tile_data_base_address
                + (if self.bg_window_tile_data_base_address == 0x8000 {
                tilenr as u16
            } else {
                (tilenr as i8 as i16 + 128) as u16
            }) * 16;

            let a0 = tileaddress + (pixely * 2);

            let (b1, b2) = (self.rbvram0(a0), self.rbvram0(a0 + 1));

            let xbit = 7 - pixelx;

            let colnr =
                if b1 & (1 << xbit) != 0 {
                    1
                } else {
                    0
                }

                    |

                if b2 & (1 << xbit) != 0 {
                    2
                } else {
                    0
                };

            self.bgprio[x] =
                if colnr == 0 { PrioType::Color0 }
                    else { PrioType::Normal };

            let color = self.palb[colnr];
            self.setcolor(x, color);
        }
    }

    fn draw_sprites(&mut self) {
        if !self.sprite_enable {
            return
        }

        for index in 0 .. 40 {
            let i = 39 - index;
            let spriteaddr = 0xFE00 + (i as u16) * 4;

            let spritey = self.rb(spriteaddr + 0) as u16 as i32 - 16;
            let spritex = self.rb(spriteaddr + 1) as u16 as i32 - 8;

            let tilenum = (self.rb(spriteaddr + 2) & (
                if self.sprite_size == 16 {
                    0xFE
                } else {
                    0xFF
                }
            )) as u16;

            let flags = self.rb(spriteaddr + 3) as usize;
            let usepal1: bool = flags & (1 << 4) != 0;
            let xflip: bool = flags & (1 << 5) != 0;
            let yflip: bool = flags & (1 << 6) != 0;
            let belowbg: bool = flags & (1 << 7) != 0;

            let line = self.line as i32;
            let sprite_size = self.sprite_size as i32;

            if line < spritey || line >= spritey + sprite_size {
                continue
            }

            if spritex < -7 || spritex >= (WIDTH as i32) {
                continue
            }

            let tiley: u16 =
                if yflip {
                    (sprite_size - 1 - (line - spritey)) as u16
                } else {
                    (line - spritey) as u16
                };

            let tileaddress = 0x8000u16 + tilenum * 16 + tiley * 2;
            let (b1, b2) = (self.rbvram0(tileaddress), self.rbvram0(tileaddress + 1));

            for x in 0 .. 8 {
                if spritex + x < 0 || spritex + x >= (WIDTH as i32) {
                    continue
                }

                let xbit = 1 << (
                    if xflip {
                        x
                    } else {
                        7 - x
                    } as u32
                );

                let colnr =
                    (if b1 & xbit != 0 {
                        1
                    } else {
                        0
                    })

                        |

                    (if b2 & xbit != 0 {
                        2
                    } else {
                        0
                    });

                if colnr == 0 {
                    continue
                }

                if belowbg && self.bgprio[(spritex + x) as usize] != PrioType::Color0 {
                    continue
                }

                let color = if usepal1 {
                    self.pal1[colnr]
                } else {
                    self.pal0[colnr]
                };

                self.setcolor((spritex + x) as usize, color);
            }
        }
    }
}