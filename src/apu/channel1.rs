use super::{envelope::Envelope, sweep::Sweep, wave::WaveDuty};

pub struct Channel1 {
    sweep: Sweep,
    wave_duty: WaveDuty,
    envelope: Envelope,
    freq_bits: u16,
    use_counter: bool,
    counter: usize,
    status: bool,
}
