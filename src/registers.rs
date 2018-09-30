use std::fmt;

/// Zero flag : set if the last operation evaluates to zero, otherwise
/// is cleared.
pub const Z_FLAG: u8 = 0b_1000_0000;
/// Substraction flag : set if the last operation was a substraction, otherwise
/// is cleared.
pub const N_FLAG: u8 = 0b_0100_0000;
/// Half-carry flag : set if the last operation had an overflow from the 3rd
/// into the 4th bit, otherwise is cleared.
pub const H_FLAG: u8 = 0b_0010_0000;
/// Carry flag : set if the last operation had an overflow from the 7th into
/// the 8th bits, otherwise is cleared.
pub const C_FLAG: u8 = 0b_0001_0000;

/// Holds the state of the Game Boy CPU's internal registers.
/// The A,F,B,C,D,E,H,L registers can be paired to produce 16 bit values as so :
/// AF, BC, DE, HL.
pub struct Registers {
    /// Accumulator register ; most of the processed data passes through it.
    pub a: u8,
    /// Flag register.
    /// The Sharp LR35902 only uses the most significant nibble.
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    /// Program counter.
    pub pc: u16,
    /// Stack pointer.
    pub sp: u16,
}

impl Registers {
    /// Create and return a new, initialized Registers instance.
    pub fn new() -> Registers {
        Registers {
            a: 0x0,
            f: 0b_0000_0000,
            b: 0x0,
            c: 0x0,
            d: 0x0,
            e: 0x0,
            h: 0x0,
            l: 0x0,
            pc: 0,
            sp: 0,
        }
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_af(&mut self, af: u16) {
        // filter the provided F value with 0xF0
        self.a = (af >> 8) as u8;
        self.f = (af & 0x00F0) as u8;
    }
    pub fn set_bc(&mut self, bc: u16) {
        self.b = (bc >> 8) as u8;
        self.c = (bc & 0x00FF) as u8;
    }
    pub fn set_de(&mut self, de: u16) {
        self.d = (de >> 8) as u8;
        self.e = (de & 0x00FF) as u8;
    }
    pub fn set_hl(&mut self, hl: u16) {
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0x00FF) as u8;
    }

    pub fn flag(&mut self, mask: u8) -> bool {
        self.f & mask != 0
    }
    pub fn set_flag(&mut self, mask: u8, value: bool) {
        match value {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
    }
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               concat!("A:{:0>2X} B:{:0>2X} C:{:0>2X} D:{:0>2X} E:{:0>2X} ",
                       "F:{:0>8b} H:{:0>2X} L:{:0>2X} SP:{:0>4X} PC:{:0>4X}"),
               self.a,
               self.b,
               self.c,
               self.d,
               self.e,
               self.f,
               self.h,
               self.l,
               self.sp,
               self.pc)
    }
}
