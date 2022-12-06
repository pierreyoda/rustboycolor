/// Duty of the Wave channel.
///
/// See: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Wave_Channel
#[derive(Clone)]
pub enum WaveDuty {
    HalfQuarter = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
}

impl WaveDuty {
    pub fn from_u8(byte: u8) -> Option<WaveDuty> {
        match byte {
            0 => Some(WaveDuty::HalfQuarter),
            1 => Some(WaveDuty::Quarter),
            2 => Some(WaveDuty::Half),
            3 => Some(WaveDuty::ThreeQuarters),
            _ => None,
        }
    }
}
