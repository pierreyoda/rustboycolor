use crate::memory::Memory;

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

/// The Audio Processing Unit of the Game Boy (Color).
pub struct Apu {
    channel: ApuChannel,
}

impl Apu {
    pub fn new(channel: ApuChannel) -> Self {
        Self { channel }
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
