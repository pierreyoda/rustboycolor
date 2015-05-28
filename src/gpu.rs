use super::memory::Memory;

/// The width of the Game Boy's screen, in pixels.
pub const SCREEN_W: usize = 160;
/// The height of the Game Boy's screen, in pixels.
pub const SCREEN_H: usize = 144;

/// The structure holding and emulating the GPU state.
pub struct Gpu;

impl Gpu {
    /// Create and return a new 'Gpu' instance.
    pub fn new() -> Gpu {
        Gpu
    }
}

impl Memory for Gpu {
    fn read_byte(&mut self, address: u16) -> u8
    { 0 }
    fn write_byte(&mut self, address: u16, byte: u8)
    { }
}
