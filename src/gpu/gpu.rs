
/// Video Ram size, 16 kb
const VIDEO_RAM_SIZE: usize = 0x8000;

/// Object Attribute Memory size, 160 bytes (4 bits per sprite at 40 sprites)
const VIDEO_OBJECT_ATTRIBUTE_MEMORY_SIZE: usize = 0xA0;

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

/// Graphic Processing Unit
///
/// This is where the all the graphics processing happens.
/// It's also called the LCD.
///
/// The GPU contains all graphic raw_pixels, which is
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
    /// period, the screen is blank and VIDEO_RAM and OAM can be accessed freely
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

    /// Background & Window tile raw_pixels base address
    ///
    /// This indicates where is the base address at which
    /// we can read bg and window tile raw_pixels.
    ///
    /// 0 -> 8800-97FF, 1 -> 8000-8FFF
    bg_window_tile_data_base_address: u16,

    /// Background tile map base address
    ///
    /// Same as before, but for the background tilemap raw_pixels
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

    /// Background-Window / OBJ palette shades raw_pixels
    ///
    /// These three registers assigns shades of grey (GameBoy LCD supports 4
    /// shades of grey) in order to be used by the BG, 
    /// Window and OBJ sprites respectively
    bg_palette_data: u8,
    obj_0_palette_data: u8,
    obj_1_palette_data: u8,

    /// Background-Window / OBJ palette colors
    ///
    /// These three internal variables stores the corresponding RGB
    /// values for the different shades of gray that stands as:
    ///
    /// 0 -> White -> 255
    /// 1 -> Light gray -> 192
    /// 2 -> Dark gray -> 96
    /// 4 -> Black -> 0
    bg_palette_colors: [u8; 4],
    obj_0_palette_colors: [u8; 4],
    obj_1_palette_colors: [u8; 4],

    /// Video RAM
    ///
    /// This is where the Background, Window and Tile raw_pixels
    /// is stored for the GPU to work with.
    ///
    /// It has two address ranges:
    ///
    /// * 8000-97FF -> Contains Tile raw_pixels
    /// * 9800-9FFF -> Background and Window raw_pixels (used indistinctly)
    video_ram: [u8; VIDEO_RAM_SIZE],

    /// Object Attribute Memory (OAM)
    ///
    /// This is where raw_pixels about the sprites is stored. It contains
    /// 160 bytes of raw_pixels, which correspond with 40 sprite attributes (4
    /// bits per sprite)
    video_object_attribute_memory: [u8; VIDEO_OBJECT_ATTRIBUTE_MEMORY_SIZE],

    bg_priority: [PrioType; WIDTH],

    /// GPU Interrupt
    ///
    /// The GPU has 2 interrupts:
    ///
    /// * When the V-Blank period starts, during this period the VIDEO_RAM is accessible
    /// *
    pub interrupt: u8,

    /// Raw pixels vector
    ///
    /// This is a list of all the calculated pixels
    /// that will be later blit into the screen (OpenGL)
    pub raw_pixels: Vec<u8>
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
            bg_palette_data: 0,
            obj_0_palette_data: 0,
            obj_1_palette_data: 1,
            bg_palette_colors: [0; 4],
            obj_0_palette_colors: [0; 4],
            obj_1_palette_colors: [0; 4],
            video_ram: [0; VIDEO_RAM_SIZE],
            video_object_attribute_memory: [0; VIDEO_OBJECT_ATTRIBUTE_MEMORY_SIZE],
            raw_pixels: vec![0; WIDTH * HEIGHT * 3], // each pixel is a RGB value, so 24 bits are needed per pixel
            bg_priority: [PrioType::Normal; WIDTH],
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
                // retrack the clock by
                self.clock -= 456;

                // advance by one line
                self.line = self.line.wrapping_add(1);

                self.check_interrupt_lyc();

                // we reach the last line, we need to change mode to vertical
                // blank to start a new screen calculation
                if self.line >= HEIGHT as u8 && self.mode != Mode::VerticalBlank {
                    self.change_mode(Mode::VerticalBlank);
                }
            }

            if self.line < HEIGHT as u8 {
                // under 80 cycles, we are still reading OAM
                if self.clock <= 80 {
                    if self.mode != Mode::OAMRead {
                        self.change_mode(Mode::OAMRead);
                    }
                // under 80 (OAM reading) + 172 (VRAM reading), we are still
                // in VRAM reading mode
                } else if self.clock <= (80 + 172) {
                    if self.mode != Mode::VRAMRead {
                        self.change_mode(Mode::VRAMRead);
                    }
                // if not, we are in horizontal blank (we finished rendering
                // one line)
                } else {
                    if self.mode != Mode::HorizontalBlank {
                        self.change_mode(Mode::HorizontalBlank);
                    }
                }
            }
        }
    }

    /// Read byte from the GPU
    ///
    /// Like the MMU, the GPU maps a range of addresses
    /// to it's internal state. You can see more info on
    /// each of the address ranges commented in the code.
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000 ... 0x9FFF => self.video_ram                     [address as usize & 0x1FFF],

            0xFE00 ... 0xFE9F => self.video_object_attribute_memory [address as usize - 0xFE00],

            // GPU Control read. we return the status of the LCD
            // as a 8 bit hex number. This is a good example on how bitwise
            // operations works. we need to set every bit from the hex number
            // and we can do it by applying the OR operator, at each position
            // of the control hex number, like this:
            // Bit 7 -> 1000 0000 (0x80)
            // Bit 6 -> 0100 0000 (0x40)
            // Bit 5 -> 0010 0000 (0x20)
            // Bit 4 -> 0001 0000 (0x10)
            // Bit 3 -> 0000 1000 (0x08)
            // Bit 2 -> 0000 0100 (0x04)
            // Bit 1 -> 0000 0010 (0x02)
            // Bit 0 -> 0000 0001 (0x01)
            0xFF40 => {
                (if self.lcd_display_enable                                 { 0x80 } else { 0 })    |
                    (if self.window_tile_map_display_base_address == 0x9C00 { 0x40 } else { 0 })    |
                    (if self.window_display_enable                          { 0x20 } else { 0 })    |
                    (if self.bg_window_tile_data_base_address == 0x8000     { 0x10 } else { 0 })    |
                    (if self.bg_tile_map_base_address == 0x9C00             { 0x08 } else { 0 })    |
                    (if self.sprite_size == 16                              { 0x04 } else { 0 })    |
                    (if self.sprite_enable                                  { 0x02 } else { 0 })    |
                    (if self.background_display_enable                      { 0x01 } else { 0 })
            },

            // GPU STAT read. Interrupts in the GPU state
            0xFF41 => {
                (if self.lyc_interrupt                  { 0x40 } else { 0 })    |
                    (if self.oam_interrupt              { 0x20 } else { 0 })    |
                    (if self.vertical_blank_interrupt   { 0x10 } else { 0 })    |
                    (if self.horizontal_blank_interrupt { 0x08 } else { 0 })    |
                    (if self.line == self.lyc           { 0x04 } else { 0 })    |

                    self.mode as u8
            },

            // the following reads maps 1:1 to the rest of raw_pixels
            // found in the GPU
            0xFF42 => self.scroll_position_y,

            0xFF43 => self.scroll_position_x,

            0xFF44 => self.line,

            0xFF45 => self.lyc,

            0xFF46 => 0, // Write only

            0xFF47 => self.bg_palette_data,

            0xFF48 => self.obj_0_palette_data,

            0xFF49 => self.obj_1_palette_data,

            0xFF4A => self.window_position_y,

            0xFF4B => self.window_position_x,

            _ => {
                println!("Invalid GPU Read {:04X}", address);
                0
            },
        }
    }

    /// Write byte to the GPU
    ///
    /// Also, like the MMU, the GPU changes it's internal state
    /// by writing some value into it. The range of addresses will
    /// perform a variety of changes, described on the code.
    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // manipulates the video ram raw_pixels. We apply the AND & operator
            // in order to map the hex address requested to our 0-indexed vector
            0x8000 ... 0x9FFF => self.video_ram[address as usize & 0x1FFF] = value,

            0xFE00 ... 0xFE9F => self.video_object_attribute_memory[address as usize - 0xFE00] = value,

            // GPU control write. see more details in the method
            0xFF40 => {
                self.handle_gpu_control(value)
            },

            // we see the proper interrupts taking them from the
            // 4 more significant bits of the value
            0xFF41 => {
                self.lyc_interrupt =                value & 0x40 == 0x40;
                self.oam_interrupt =                value & 0x20 == 0x20;
                self.vertical_blank_interrupt =     value & 0x10 == 0x10;
                self.horizontal_blank_interrupt =   value & 0x08 == 0x08;
            },

            // some 1:1 mapped value settings to follow
            0xFF42 => self.scroll_position_y = value,

            0xFF43 => self.scroll_position_x = value,

            0xFF44 => {},

            0xFF45 => self.lyc = value,

            0xFF46 => panic!("0xFF46 should be handled by MMU"),

            // these three 1:1 mapped, but they also need to calculate
            // the RGB pixel color to be pushed to the screen
            0xFF47 => {
                self.bg_palette_data = value;
                self.update_palette_colors();
            },

            0xFF48 => {
                self.obj_0_palette_data = value;
                self.update_palette_colors();
            },

            0xFF49 => {
                self.obj_1_palette_data = value;
                self.update_palette_colors();
            },

            0xFF4A => self.window_position_y = value,

            0xFF4B => self.window_position_x = value,

            _ => {
                println!("Invalid GPU write {:04X}", address)
            },
        }
    }

    fn check_interrupt_lyc(&mut self) {
        if self.lyc_interrupt && self.line == self.lyc {
            self.interrupt |= 0x02;
        }
    }

    /// Changes GPU mode
    /// 
    /// This method will change the current GPU mode and perform
    /// each mode operations as follow:
    /// 
    /// * Horizontal blank: render line
    /// * Vertical blank: perform interrupt
    /// * OAM read: perform interrupt
    /// * VRAM read: nothing to do
    fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;

        let interrupt = match self.mode {
            Mode::HorizontalBlank => {
                self.render_line();
                self.horizontal_blank_interrupt
            },
            Mode::VerticalBlank => {
                self.interrupt |= 0x01;
                self.vertical_blank_interrupt
            },
            Mode::OAMRead => self.oam_interrupt,
            _ => false,
        };

        if interrupt {
            self.interrupt |= 0x02;
        }
    }

    fn read_byte_from_video_ram(&self, address: u16) -> u8 {
        self.video_ram[address as usize & 0x1FFF]
    }

    /// Handle the GPU STAT / Control instruction
    ///
    /// This operation sets the GPU status, changing several internal
    /// variables and registers that will later be used to perform
    /// a variety of operations. See each property documentation for more info
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

        for v in self.raw_pixels.iter_mut() {
            *v = 255;
        }
    }

    fn update_palette_colors(&mut self) {
        for i in 0 .. 4 {
            self.bg_palette_colors[i]       = GPU::get_monochrome_rgb_value(self.bg_palette_data, i);
            self.obj_0_palette_colors[i]    = GPU::get_monochrome_rgb_value(self.obj_0_palette_data, i);
            self.obj_1_palette_colors[i]    = GPU::get_monochrome_rgb_value(self.obj_1_palette_data, i);
        }
    }

    fn get_monochrome_rgb_value(value: u8, index: usize) -> u8 {
        match (value >> 2*index) & 0x03 {
            0 => 255,
            1 => 192,
            2 => 96,
            _ => 0
        }
    }

    /// Render a line
    ///
    /// In this method we render a line by first
    /// drawing the background/window and then drawing
    /// the sprites on top of that
    fn render_line(&mut self) {
        // reset all pixels and bg priority in for the current line
        // (current line is a class property, not showed here)
        for x in 0 .. WIDTH {
            self.calculate_pixel(x, 255);
            self.bg_priority[x] = PrioType::Normal;
        }

        self.draw_background();
        self.draw_sprites();
    }

    /// Calculates a pixel
    ///
    /// Each pixel has 3 color components, which are RGB as
    /// pero OpenGL pixel format (U8U8U8)
    fn calculate_pixel(&mut self, position_x: usize, color: u8) {
        let position_y = self.line as usize;

        self.raw_pixels[position_y * WIDTH * 3 + position_x * 3 + 0] = color;
        self.raw_pixels[position_y * WIDTH * 3 + position_x * 3 + 1] = color;
        self.raw_pixels[position_y * WIDTH * 3 + position_x * 3 + 2] = color;
    }

    /// Draws the background and window
    /// 
    /// This is the function called before drawing sprites, and will
    /// draw both the background and window layer on the screen. More
    /// details in the implementation
    fn draw_background(&mut self) {
        let draw_background = self.background_display_enable;

        // first we calculate the window position.
        let window_position_y =
            if !self.window_display_enable || !self.background_display_enable {
                -1
            } else {
                // if the window is being draw, the position aligned
                // with the current line beign raw
                self.line as i32 - self.window_position_y as i32
            };

        // if no window and bg are displayed, we return to the caller
        if window_position_y < 0 && draw_background == false {
            return;
        }

        // calculate the window tile
        let window_tile_y = (window_position_y as u16 >> 3) & 31;

        // calculate the background Y position by adding
        // the current line to the current scroll Y position
        let background_y = self.scroll_position_y.wrapping_add(self.line);

        let background_tile_y = (background_y as u16 >> 3) & 31;

        for x in 0 .. WIDTH {
            self.draw_background_line(
                x,
                draw_background,
                background_tile_y,
                background_y,
                window_tile_y,
                window_position_y
            )
        }
    }

    fn draw_background_line(
        &mut self,
        x: usize,
        draw_background: bool,
        background_tile_y: u16,
        background_y: u8,
        window_tile_y: u16,
        window_position_y: i32
    ) {
        let window_position_x = - ((self.window_position_x as i32) - 7) + (x as i32);
        let background_x = self.scroll_position_x as u32 + x as u32;

        // calculate tile map base addresses
        // and positions inside the VRAM.
        // these values will be used for:
        // 1. calculate the tile number from VRAM memory
        // 2. calculate tile address to fetch raw data from VRAM
        let (
            tile_map_base_address,
            tile_y,
            tile_x,
            pixel_y,
            pixel_x
        ) =
        if window_position_y >= 0 && window_position_x >= 0 {
            (
                self.window_tile_map_display_base_address,
                window_tile_y,
                (window_position_x as u16 >> 3),
                window_position_y as u16 & 0x07,
                window_position_x as u8 & 0x07
            )
        } else if draw_background {
            (
                self.bg_tile_map_base_address,
                background_tile_y,
                (background_x as u16 >> 3) & 31,
                background_y as u16 & 0x07,
                background_x as u8 & 0x07
            )
        } else {
            return;
        };

        let tile_number: u8 = self.read_byte_from_video_ram(tile_map_base_address + tile_y * 32 + tile_x);

        let tile_address =

        self.bg_window_tile_data_base_address +
            (
                if self.bg_window_tile_data_base_address == 0x8000 {
                    tile_number as u16
                } else {
                    (tile_number as i8 as i16 + 128) as u16
                }
            ) * 16;

        let a0 = tile_address + (pixel_y * 2);

        let (b1, b2) = (
            self.read_byte_from_video_ram(a0),
            self.read_byte_from_video_ram(a0 + 1)
        );

        let xbit = 7 - pixel_x;

        let color_number =
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

        self.bg_priority[x] =
            if color_number == 0 {
                PrioType::Color0
            } else {
                PrioType::Normal
            };

        let color = self.bg_palette_colors[color_number];

        self.calculate_pixel(x, color);
    }

    fn draw_sprites(&mut self) {
        if !self.sprite_enable {
            return
        }

        for index in 0 .. 40 {
            let i = 39 - index;
            let sprite_address = 0xFE00 + (i as u16) * 4;

            let spritey = self.read_byte(sprite_address + 0) as u16 as i32 - 16;
            let spritex = self.read_byte(sprite_address + 1) as u16 as i32 - 8;

            let tile_number =
                (
                    self.read_byte(sprite_address + 2) &

                    // sprites can be 16 or 8 sized
                    (if self.sprite_size == 16 {
                        0xFE
                    } else {
                        0xFF
                    })

                ) as u16;

            // we read this sprite flags in order to determine some
            // rendering options, that can be set by game programmers.
            // this is done by bit shifting the options stored (as hex)
            // and converting to bools we will be using
            let flags = self.read_byte(sprite_address + 3) as usize; // flags are
            let useobj_1_palette_colors: bool = flags & (1 << 4) != 0;
            let xflip: bool = flags & (1 << 5) != 0;
            let yflip: bool = flags & (1 << 6) != 0;
            let belowbg: bool = flags & (1 << 7) != 0;

            let line = self.line as i32;
            let sprite_size = self.sprite_size as i32;

            // ignore some obvious sprite limits
            if line < spritey || line >= spritey + sprite_size {
                continue
            }

            if spritex < -7 || spritex >= (WIDTH as i32) {
                continue
            }

            // calculate tile Y position by checking
            // if the sprite is Y flipped
            let tile_y: u16 =
                if yflip {
                    (sprite_size - 1 - (line - spritey)) as u16
                } else {
                    (line - spritey) as u16
                };

            // calculate base address where data for this sprite
            // is stored
            let tile_address = 0x8000u16 + tile_number * 16 + tile_y * 2;

            let (b1, b2) = (
                self.read_byte_from_video_ram(tile_address),
                self.read_byte_from_video_ram(tile_address + 1)
            );

            for x in 0 .. 8 {
                // check sprite pixel still shows on the screen
                if spritex + x < 0 || spritex + x >= (WIDTH as i32) {
                    continue
                }

                // calculate pixel position based
                // on X flip state
                let xbit = 1 << (
                    if xflip {
                        x
                    } else {
                        7 - x
                    } as u32
                );

                // calculate color number to be
                // fetched from the palette before
                // calculating the pixel
                let color_number =
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

                // if color is 0 it means the pixel is not visible
                if color_number == 0 {
                    continue
                }

                // here we first check the belowbg sprite property (set by game programmers)
                // and then this pixel position's bg priority (also set by programmers)
                // if the background takes priority, the pixel is not calculate
                if belowbg && self.bg_priority[(spritex + x) as usize] != PrioType::Color0 {
                    continue
                }


                let color = if useobj_1_palette_colors {
                    self.obj_1_palette_colors[color_number]
                } else {
                    self.obj_0_palette_colors[color_number]
                };

                self.calculate_pixel((spritex + x) as usize, color);
            }
        }
    }
}