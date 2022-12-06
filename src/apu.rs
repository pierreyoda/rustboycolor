use crate::{cpu::CycleType, memory::Memory};

mod channel1;
mod channel2;
mod channel3;
mod channel4;
mod envelope;
mod sweep;
mod wave;

#[derive(Clone, Copy)]
enum Volume {
    Vol0 = 0x00,
    Vol1 = 0x01,
    Vol2 = 0x02,
    Vol3 = 0x03,
    Vol4 = 0x04,
    Vol5 = 0x05,
    Vol6 = 0x06,
    Vol7 = 0x07,
}

impl Volume {
    pub fn from_u8(byte: u8) -> Option<Volume> {
        match byte {
            0x00 => Some(Volume::Vol0),
            0x01 => Some(Volume::Vol1),
            0x02 => Some(Volume::Vol2),
            0x03 => Some(Volume::Vol3),
            0x04 => Some(Volume::Vol4),
            0x05 => Some(Volume::Vol5),
            0x06 => Some(Volume::Vol6),
            0x07 => Some(Volume::Vol7),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ChannelsFlag {
    Channel1 = 1 << 0,
    Channel2 = 1 << 1,
    Channel3 = 1 << 2,
    Channel4 = 1 << 3,
}

pub enum ApuChannel {
    /// Quadrangular wave patterns with sweep and envelope functions (CH1).
    SweepAndEnvelope,
    /// Quadrangular wave patterns with envelope functions (CH2).
    Envelop,
    /// Voluntary wave patterns from wave RAM (CH3).
    Wave,
    /// White noise with an envelope function (CH4).
    Noise,
    /// Mixed channel.
    Mixed,
}

/// The Audio Processing Unit (APU) of the Game Boy (Color).
pub struct Apu {
    enabled: bool,
    term1_volume: Volume,
    term1_vin: bool,
    term1_channels: ChannelsFlag,
    term2_volume: Volume,
    term2_vin: bool,
    term2_channels: ChannelsFlag,
    channel: ApuChannel,
    cycles: CycleType,
}

impl Apu {
    /// Create and define a new `Apu` instance.
    pub fn new(channel: ApuChannel) -> Self {
        Self {
            enabled: true,
            channel,
            cycles: 4096,
        }
    }
}

impl Memory for Apu {
    fn read_byte(&mut self, address: u16) -> u8 {
        todo!()
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        todo!()
    }
}
