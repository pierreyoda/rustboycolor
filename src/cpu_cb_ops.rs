use super::cpu::*;
use super::memory::Memory;
use super::registers::{Registers, Z_FLAG_MASK, N_FLAG_MASK, H_FLAG_MASK,
    C_FLAG_MASK};

//
// --- Helper functions ---
//

// Swap the byte's nibbles, reset the NHC flags and set the Z flag in f.
fn swap(f: &mut u8, val: &mut u8) {
    *f = 0;
    let x1 = (*val & 0x0F) << 4;
    let x2 = (*val >> 4) & 0x0F;
    *val = x1 | x2;
    if *val == 0 {
        *f |= Z_FLAG_MASK;
    }
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
    pub fn SWAP_r_b(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.b); 2 }
    pub fn SWAP_r_c(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.c); 2 }
    pub fn SWAP_r_d(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.d); 2 }
    pub fn SWAP_r_e(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.e); 2 }
    pub fn SWAP_r_h(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.h); 2 }
    pub fn SWAP_r_l(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.l); 2 }
    pub fn SWAP_r_a(&mut self) -> CycleType { swap(&mut self.regs.f, &mut self.regs.a); 2 }
    // SWAP_HLm : same but for (HL)
    pub fn SWAP_HLm(&mut self) -> CycleType {
        let address = (self.regs.h as u16) << 8 + self.regs.l as u16;
        let mut temp_byte = self.mem.read_byte(address);
        swap(&mut self.regs.f, &mut temp_byte);
        self.mem.write_byte(address, temp_byte);
        4
    }
}
