#[derive(Clone, Copy)]
enum SweepTime {
    None = 0,
    Div1 = 1,
    Div2 = 2,
    Div3 = 3,
    Div4 = 4,
    Div5 = 5,
    Div6 = 6,
    Div7 = 7,
}

impl SweepTime {
    pub fn from_u8(byte: u8) -> Option<SweepTime> {
        match byte {
            0 => Some(SweepTime::None),
            1 => Some(SweepTime::Div1),
            2 => Some(SweepTime::Div2),
            3 => Some(SweepTime::Div3),
            4 => Some(SweepTime::Div4),
            5 => Some(SweepTime::Div5),
            6 => Some(SweepTime::Div6),
            7 => Some(SweepTime::Div7),
            _ => None,
        }
    }
}

/// Determines the Frequency Sweep of an APU channel.
///
/// See: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Frequency_Sweep
#[derive(Clone)]
pub struct Sweep {
    time: SweepTime,
    increasing: bool,
    shift: u8,
}

impl Sweep {
    pub fn new() -> Sweep {
        Sweep {
            time: SweepTime::None,
            increasing: false,
            shift: 0,
        }
    }

    pub fn read(&self) -> u8 {
        const MASK: u8 = 0x80;
        MASK | ((self.time as u8) << 4) | if self.increasing { 1 << 3 } else { 0 } | self.shift
    }

    pub fn write(&mut self, byte: u8) {
        let sweep_time_value = (byte >> 4) & 0x07;
        self.time = SweepTime::from_u8(sweep_time_value).expect(&format!(
            "apu::Sweep.write_reg({:0>2X}): invalid SweepTime value",
            sweep_time_value,
        ));
        self.increasing = byte & (1 << 3) != 0;
        self.shift = byte & 0x07;
    }
}
