use super::memory::Memory;

/// The Game Boy (Color)'s Memory Management Unit, interfacing between
/// its CPU and the different memory components (RAM, ROM banks...).
/// Responsible for switching between the different ROM and RAM banks.
pub struct MMU;

// MMU implements the Memory trait to provide transparent interfacing
// with the CPU.
impl Memory for MMU {
    fn read_byte(address: u16, byte: u8) {

    }
    fn write_byte(address: u16) -> u8 {
        0x0
    }
}
