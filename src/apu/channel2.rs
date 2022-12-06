use super::{envelope::Envelope, wave::WaveDuty};

pub struct Channel2 {
    wave_duty: WaveDuty,
    envelope: Envelope,
    freq_bits: u16,
    use_counter: bool,
    counter: usize,
    status: bool,
}

impl Default for Channel2 {
    fn default() -> Self {
        Self {
            wave_duty: WaveDuty::HalfQuarter,
            envelope: Envelope::new(),
            freq_bits: 0,
            use_counter: false,
            counter: 0,
            status: false,
        }
    }
}

impl Channel2 {
    pub fn reset(&mut self) {
        *self = Channel2::default();
    }

    pub fn tick(&mut self) {
        if self.use_counter && self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.status = false;
            }
        }
    }

    pub fn read_register_1(&self) -> u8 {
        const REG1_MASK: u8 = 0x3F;

        REG1_MASK | ((self.wave_duty as u8) << 6)
    }

    pub fn write_register_1(&mut self, value: u8) {
        let wave_duty_value = (value >> 6) & 0x03;
        self.wave_duty = WaveDuty::from_u8(wave_duty_value).expect(&format!(
            "apu::Channel2.write_register1({}): invalid wave duty value {:0>2X}",
            value, wave_duty_value
        ));
        self.counter = 64 - (value & 0x3f) as usize;
    }

    pub fn write_register_3(&mut self, value: u8) {
        self.freq_bits = (self.freq_bits & 0x700) | value as u16;
    }

    pub fn read_register_4(&self) -> u8 {
        const REG4_MASK: u8 = 0xBF;

        REG4_MASK | if self.use_counter { 1 << 6 } else { 0 }
    }

    pub fn write_register_4(&mut self, byte: u8) {
        self.status = byte & (1 << 7) != 0;
        self.use_counter = byte & (1 << 6) != 0;
        self.freq_bits = (self.freq_bits & 0xff) | ((byte as u16) << 8);
        if self.status && self.counter == 0 {
            self.counter = 64;
        }
    }
}
