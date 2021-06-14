use super::blip_buf::BlipBuf;

const WAVE_PATTERN : [[i32; 8]; 4] = [[-1,-1,-1,-1,1,-1,-1,-1],[-1,-1,-1,-1,1,1,-1,-1],[-1,-1,1,1,1,1,-1,-1],[1,1,1,1,-1,-1,1,1]];
const CLOCKS_PER_SECOND : u32 = 1 << 22;
const OUTPUT_SAMPLE_COUNT : usize = 2000;

pub struct Audio {
    on: bool,
    channel1: ToneSweepChannel,
}

pub trait AudioPlayer : Send {
    fn play(&mut self, left_channel: &[f32], right_channel: &[f32]);
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
    sweep_frequency: u16,
    wave_duty: u8,
    sound_length_next: u8,
    sound_length: u8,
    frequency_lsb: u8, // separated for clarity
    frequency_msb: u8,
    current_frequency: u16,
    length_enabled: bool,
    trigger_event: bool
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
            sweep_frequency: 0,
            wave_duty: 0,
            sound_length_next: 0,
            sound_length: 0,
            frequency_lsb: 0,
            frequency_msb: 0,
            current_frequency: 0,
            length_enabled: false,
            trigger_event: false
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
                self.length_enabled = (value & 64) == 64;
                self.trigger_event = (value & 128) == 128;
            }
            _ => panic!("Unhandled audio write: {:04X} - {:08b}", address, value)
        }
    }

    pub fn step(&mut self)
    {
        self.current_frequency = ((self.frequency_lsb << 3) as u16) | self.frequency_msb as u16;

        let period = if self.current_frequency > 2048 {
            0
        } else {
            (2048 - (self.current_frequency as u32)) * 4
        };

        // handle trigger
        if self.trigger_event {
            if self.sweep_time != 0 {
                let offset = self.current_frequency << self.sweep_shift;

                if self.sweep_direction {
                    if self.current_frequency <= offset
                    {
                        self.sweep_frequency = 0;
                    } else {
                        self.sweep_frequency -= offset;
                    }
                } else {
                    if self.sweep_frequency >= 2048 - offset {
                        self.sweep_frequency = 2048
                    } else {
                        self.sweep_frequency += offset;
                    }
                }
            }
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