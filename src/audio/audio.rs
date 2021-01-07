pub struct Audio {
    on: bool,
    channel1: ToneSweepChannel,
}

struct VolumeEnvelope {
    direction: bool,
    initial_volume: u8
}

struct ToneSweepChannel {
    envelope: VolumeEnvelope,
    sweep_shift: u8,
    sweep_direction: bool,
    sweep_time: u8,
    wave_duty: u8,
    sound_length_next: u8,
    sound_length: u8,
    frequency_lsb: u8, // separated for clarity
    frequency_msb: u8,
    length_enabled: bool
}

impl VolumeEnvelope
{
    pub fn new() -> VolumeEnvelope {
        VolumeEnvelope {
            direction: false,
            initial_volume: 0
        }
    }
}

impl ToneSweepChannel
{
    pub fn new() -> ToneSweepChannel {
        ToneSweepChannel {
            envelope: VolumeEnvelope::new(),
            sweep_shift: 0,
            sweep_direction: false,
            sweep_time: 0,
            wave_duty: 0,
            sound_length_next: 0,
            sound_length: 0,
            frequency_lsb: 0,
            frequency_msb: 0,
            length_enabled: false,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8)
    {
        match address {
            0xFF10 => {
                self.sweep_shift = value & 0x7;
                self.sweep_direction = (value & 0x8) == 0x8;
                self.sweep_time = (value >> 4) & 0x7;
            },
            0xFF11 => {
                self.wave_duty = value >> 6;
                self.sound_length = 63 - (value & 0x63)
            },
            0xFF12 => {
                self.envelope.initial_volume = value << 4;
                self.envelope.direction = (value & 0x8) == 0x8;
            },
            0xFF13 => {
                self.frequency_lsb = value
            },
            0xFF14 => {
                self.frequency_msb = value & 0x7;
                self.length_enabled = (value & 0x40) == 0x40;

                let trigger = (value & 0x80) == 0x80;

                if trigger {
                    // trigger event
                }
            }
            _ => panic!("Unhandled audio write: {:04X} - {:08b}", address, value)
        }
    }
}

impl Audio {
    pub fn new() -> Audio
    {
        Audio {
            on: true,
            channel1: ToneSweepChannel::new()
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8)
    {
        match address {
            0xFF10 ..= 0xFF14 => self.channel1.write_byte(address, value),
            0xFF1A => self.on = false,
            0xFF24 => (), // Implement!,
            0xFF25 => (), // Implement!
            0xFF26 => self.on = value & value == 0x80,
            _ => ()
        }
    }
}