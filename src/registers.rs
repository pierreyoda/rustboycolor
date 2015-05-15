/// Zero flag : set if the last operation evaluates to zero, otherwise
/// is cleared.
pub const Z_FLAG_MASK: u8 = 0b_1000_0000;
/// Substraction flag : set if the last operation was a substraction, otherwise
/// is cleared.
pub const N_FLAG_MASK: u8 = 0b_0100_0000;
/// Half-carry flag : set if the last operation had an overflow from the 3rd
/// into the 4th bit, otherwise is cleared.
pub const H_FLAG_MASK: u8 = 0b_0010_0000;
/// Carry flag : set if the last operation had an overflow from the 7th into
/// the 8th bits, otherwise is cleared.
pub const C_FLAG_MASK: u8 = 0b_0001_0000;

/// Holds the state of the Game Boy CPU's internal registers.
pub struct Registers {
    pub a : u8,
    pub b : u8,
    pub c : u8,
    pub d : u8,
    pub e : u8,
    pub h : u8,
    pub l : u8,
    /// Flag register.
    pub f : u8,
    /// Program counter.
    pub pc: u16,
    /// Stack pointer.
    pub sp: u16,
}
