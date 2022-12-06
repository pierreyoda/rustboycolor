#[derive(Clone, Copy)]
enum Volume {
    None = 0,
    Full = 1,
    Half = 2,
    Quarter = 3,
}

impl Volume {
    pub fn from_u8(byte: u8) -> Option<Volume> {
        match byte {
            0 => Some(Volume::None),
            1 => Some(Volume::Full),
            2 => Some(Volume::Half),
            3 => Some(Volume::Quarter),
            _ => None,
        }
    }
}

pub struct Channel3 {
    wave_ram: [u8; 16],
    enabled: bool,
    volume: Volume,
    freq_bits: u16,
    use_counter: bool,
    counter: usize,
    status: bool,
}

impl Default for Channel3 {
    fn default() -> Self {
        Channel3 {
            wave_ram: [0x00; 16],
            enabled: false,
            volume: Volume::None,
            freq_bits: 0,
            use_counter: false,
            counter: 0,
            status: false,
        }
    }
}

impl Channel3 {
    pub fn reset(&mut self) {
        self.enabled = false;
        self.volume = Volume::None;
        self.use_counter = false;
        self.counter = 0;
        self.status = false;
    }

    pub fn tick(&mut self) {
        if self.use_counter && self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.status = false;
            }
        }
    }

    pub fn read_wave_ram(&self, relative_address: u16) -> u8 {
        self.wave_ram[relative_address as usize]
    }

    pub fn write_wave_ram(&mut self, relative_address: u16, value: u8) {
        self.wave_ram[relative_address as usize] = value;
    }

    pub fn read_register_0(&self) -> u8 {
        const REG0_MASK: u8 = 0x7f;

        REG0_MASK | if self.enabled { 1 << 7 } else { 0 }
    }

    pub fn write_register_0(&mut self, value: u8) {
        self.enabled = value & (1 << 7) != 0;
    }

    pub fn write_register_1(&mut self, value: u8) {
        self.counter = 256 - value as usize;
    }

    pub fn read_register_2(&self) -> u8 {
        const REG2_MASK: u8 = 0x9f;

        REG2_MASK | ((self.volume as u8) << 5)
    }

    pub fn write_register_2(&mut self, value: u8) {
        self.volume = Volume::from_u8((value >> 5) & 0x03).unwrap();
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.freq_bits = (self.freq_bits & 0x700) | value as u16;
    }

    pub fn read_register_4(&self) -> u8 {
        const REG4_MASK: u8 = 0xbf;

        REG4_MASK | if self.use_counter { 1 << 6 } else { 0 }
    }

    pub fn write_register_4(&mut self, value: u8) {
        self.status = value & (1 << 7) != 0;
        self.use_counter = value & (1 << 6) != 0;
        self.freq_bits = (self.freq_bits & 0xff) | ((value as u16) << 8);
        if self.status && self.counter == 0 {
            self.counter = 256;
        }
    }
}
