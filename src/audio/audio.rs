pub struct Audio {
    on: bool,
    channel1: ToneSweepChannel,
}

struct VolumeEnvelope {
    direction: bool,
    initial_volume: u8
}

struct ToneSweepChannel {
    envelope: VolumeEnvelope
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
            envelope: VolumeEnvelope::new()
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8)
    {
        match address {
            0xFF12 => {
                self.envelope.initial_volume = value << 4;
                self.envelope.direction = value & value == 0x8;
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
            0xFF24 => println!("Ignoring channel volume, assuming 100% both channels"),
            0xFF25 => println!("Ignoring output channel"),
            0xFF26 => self.on = value & value == 0x80,
            _ => ()
        }
    }
}