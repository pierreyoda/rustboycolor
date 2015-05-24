use super::cpu::*;
use super::memory::Memory;
use super::registers::{Registers, Z_FLAG_MASK, N_FLAG_MASK, H_FLAG_MASK,
    C_FLAG_MASK};

//
// --- Helper macros ---
//

// Swap the byte's nibbles, reset the NHC flags and set the Z flag in f.
macro_rules! swap {
    ($s: ident, $x: expr) => ({
        $s.regs.f = 0x0;
        let x1 = ($x & 0x0F) << 4;
        let x2 = ($x >> 4) & 0x0F;
        $x = x1 | x2;
        if $x == 0 {
            $s.regs.f |= Z_FLAG_MASK;
        }
    })
}

// The CB-prefixed opcodes are implemented in this crate for better clarity in
// the code. Notations used :
// - (X) means the value stored in memory at the X address
#[allow(non_snake_case)]
impl<M> Cpu<M> where M: Memory {

    //
    // --- SWAP ---
    //

    // SWAP_r_b : swap register X's nibbles, reset NHC flags and set Z flag
    pub fn SWAP_r_b(&mut self) -> CycleType { swap!(self, self.regs.b); 2 }
    pub fn SWAP_r_c(&mut self) -> CycleType { swap!(self, self.regs.c); 2 }
    pub fn SWAP_r_d(&mut self) -> CycleType { swap!(self, self.regs.d); 2 }
    pub fn SWAP_r_e(&mut self) -> CycleType { swap!(self, self.regs.e); 2 }
    pub fn SWAP_r_h(&mut self) -> CycleType { swap!(self, self.regs.h); 2 }
    pub fn SWAP_r_l(&mut self) -> CycleType { swap!(self, self.regs.l); 2 }
    pub fn SWAP_r_a(&mut self) -> CycleType { swap!(self, self.regs.a); 2 }
    // SWAP_HLm : same but for (HL)
    pub fn SWAP_HLm(&mut self) -> CycleType {
        let address = (self.regs.h as u16) << 8 + self.regs.l as u16;
        let mut temp_byte = self.mem.read_byte(address);
        swap!(self, temp_byte);
        self.mem.write_byte(address, temp_byte);
        4
    }
}
