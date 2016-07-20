/// GameBoy Timer
///
/// The GameBoy provides with game developers with a
/// time-based source in order to introduce animations,
/// periodic actions, etc. in their games
pub struct Timer {
    /// Divider
    ///
    /// Counts up at a fixed rate of 16384Hz
    /// Will reset to 0 when it reaches overflow
    /// (255 as per 8 bit register).
    ///
    divider: u8,

    /// Counter
    ///
    /// Counts up at a up to 4 programmable speeds
    /// (based on the divider). When it reaches overflow,
    /// an interrupt is triggered, and also the value
    /// returned in overflow state (modulo) can be configured
    counter: u8,

    /// Modulo
    ///
    /// Value to set in the counter when it reaches
    /// the overflow
    modulo: u8,

    /// Enabled
    ///
    /// Whether the timer is enabled or not
    enabled: bool,

    /// Step
    ///
    /// Speed at the counter is stepping
    step: u32,

    /// Internal counter
    ///
    /// This is used to store ticks from the CPU and
    /// calculate real counter value for the timer
    internal_counter: u32,

    /// Internal divider
    ///
    /// This is used in order to emulate the 16384Hz
    /// of divider rate, which is in fact used for
    /// the internal counter and finally for the real
    /// counter value (and of course the real divider!)
    internal_divider: u32,

    pub interrupt: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            enabled: false,
            step: 256,
            internal_counter: 0,
            internal_divider: 0,
            interrupt: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.divider,

            0xFF05 => self.counter,

            0xFF06 => self.modulo,

            // returns whether the timer is enabled
            // and at which speed is counting up
            0xFF07 => {
                let enable = if self.enabled { 0x4 } else { 0 };
                let step = match self.step {
                    16      => 1,
                    64      => 2,
                    256     => 3,
                    _       => 0,
                };

                return enable | step;
            }
            _ => panic!("Invalid timer read: {:4X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                self.divider = 0;
            },

            0xFF05 => {
                self.counter = value;
            },

            0xFF06 => {
                self.modulo = value;
            },

            // sets the timer control flags
            // bit 2: enabled or not
            // bit 0-1: speed for the counter
            0xFF07 => {
                self.enabled = value & 0x4 != 0;

                // these step values are based on the divider:
                // divider fixed rate: 256 (cpu ticks) - 16384 Hz
                self.step = match value & 0x3 {
                    1 => 16,    // divided by 16
                    2 => 64,    // divided by 8
                    3 => 256,   // divided by 1
                    _ => 1024,  // divided by 1/4
                };
            },
            _ => panic!("Invalid timer write: {:4X}", address),
        };
    }

    /// Steps the timer
    ///
    /// It takes the current CPU ticks in order to
    /// emulate the divider clock speed (used as a base
    /// for the counter)
    pub fn step(&mut self, ticks: u32) {
        self.internal_divider += ticks;

        // 256 is the time it takes the CPU to reach
        // the divider fixed rate (16384)
        while self.internal_divider >= 256 {
            // add one to the real divider
            self.divider = self.divider.wrapping_add(1);

            // retrack by same amount of cycles to wait
            self.internal_divider -= 256;
        }

        if self.enabled {
            // set the internal counter to current CPU ticks
            self.internal_counter += ticks;

            // when the CPU ticks reach the required step (which
            // is set by the programmer), we add one to the counter
            while self.internal_counter >= self.step {
                self.counter = self.counter.wrapping_add(1);

                // if the counter overflows, we set it's value
                // to the modulo value, and also trigger and interrupt
                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.interrupt |= 0x04;
                }

                // retract the internal counter
                self.internal_counter -= self.step;
            }
        }
    }
}