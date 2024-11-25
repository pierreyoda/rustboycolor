use super::envelope::Envelope;

pub struct Channel4 {
    envelope: Envelope,
    noise_opt: u8,
    use_counter: bool,
    counter: usize,
    status: bool,
}

impl Default for Channel4 {
    fn default() -> Self {
        Self {
            envelope: Envelope::new(),
            noise_opt: 0,
            use_counter: false,
            counter: 0,
            status: false,
        }
    }
}

impl Channel4 {
    pub fn reset(&mut self) {
        *self = Channel4::default();
    }

    pub fn tick(&mut self) {
        if self.use_counter && self.counter > 0 {
            self.counter -= 1;

            if self.counter == 0 {
                self.status = false;
            }
        }
    }

    pub fn write_reg1(&mut self, value: u8) {
        self.counter = 64 - (value & 0x3f) as usize;
    }

    pub fn read_reg3(&self) -> u8 {
        self.noise_opt
    }

    pub fn write_reg3(&mut self, value: u8) {
        self.noise_opt = value
    }

    pub fn read_reg4(&self) -> u8 {
        const REG4_MASK: u8 = 0xbf;

        REG4_MASK | if self.use_counter { 1 << 6 } else { 0 }
    }

    pub fn write_reg4(&mut self, value: u8) {
        self.status = value & (1 << 7) != 0;
        self.use_counter = value & (1 << 6) != 0;
        if self.status && self.counter == 0 {
            self.counter = 64;
        }
    }
}
