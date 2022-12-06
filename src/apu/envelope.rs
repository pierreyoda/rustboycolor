pub struct Envelope {
    volume: u8,
    increasing: bool,
    length: u8,
}

/// The envelope of an APU `Volume`.
///
/// See: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Volume_Envelope
impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            volume: 0,
            increasing: false,
            length: 0,
        }
    }

    pub fn read(&self) -> u8 {
        (self.volume << 4) | if self.increasing { 1 << 3 } else { 0 } | self.length
    }

    pub fn write(&mut self, byte: u8) {
        self.volume = (byte >> 4) & 0x0f;
        self.increasing = byte & (1 << 3) != 0;
        self.length = byte & 0x07;
    }
}
