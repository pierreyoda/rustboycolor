use super::memory::Memory;

/// The Game Boy (Color)'s Memory Management Unit, interfacing between
/// its CPU and the different memory banks (RAM, ROM...).
pub struct MMU;

// MMU implements the Memory trait to provide transparent interfacing
// with the CPU.
//impl Memory for MMU {
//}
