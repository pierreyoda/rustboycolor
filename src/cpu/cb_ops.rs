use super::{Cpu, CycleType};
use memory::Memory;
use mmu::MemoryManagementUnit;
use registers::{Z_FLAG, N_FLAG, H_FLAG, C_FLAG};

//
// --- Helper and implementation macros ---
// TODO : use more methods
//

macro_rules! impl_rot_reg {
    ($s: ident, $rotate_method: ident, $x: ident) => ({
        let v = $s.regs.$x;
        $s.regs.$x = $s.$rotate_method(v);
        return 2;
    })
}

macro_rules! sla {
    ($s: ident, $v: expr) => ({
        let carry = ($v & 0x80) == 0x80;
        let result = $v << 1;
        $s.regs.f = 0;
        $s.regs.set_flag(Z_FLAG, result == 0x0);
        $s.regs.set_flag(C_FLAG, carry);
        $v = result;
    })
}
macro_rules! sra {
    ($s: ident, $v: expr) => ({
        let carry = ($v & 0x01) == 0x01;
        let result = ($v >> 1) | ($v & 0x80);
        $s.regs.f = 0;
        $s.regs.set_flag(Z_FLAG, result == 0x0);
        $s.regs.set_flag(C_FLAG, carry);
        $v = result;
    })
}
macro_rules! srl {
    ($s: ident, $v: expr) => ({
        let carry  = ($v & 0x01) == 0x01;
        let result = $v >> 1;
        $s.regs.f = 0;
        $s.regs.set_flag(Z_FLAG, result == 0x0);
        $s.regs.set_flag(C_FLAG, carry);
        $v = result;
    })
}

// Swap the byte's nibbles, reset the NHC flags and set the Z flag.
macro_rules! swap {
    ($s: ident, $x: expr) => ({
        let x1 = ($x & 0x0F) << 4;
        let x2 = ($x >> 4) & 0x0F;
        $x = x1 | x2;
        $s.regs.f = if $x == 0 { Z_FLAG } else { 0 };
    })
}

macro_rules! impl_BIT_b_r_x {
    ($s: ident, $b: expr, $x: ident) => ({
        let bit = $s.regs.$x & (1 << $b);
        $s.regs.set_flag(N_FLAG, false);
        $s.regs.set_flag(H_FLAG, true);
        $s.regs.set_flag(Z_FLAG, bit == 0b0);
        return 2;
    })
}
macro_rules! impl_BIT_b_HLm {
    ($s: ident, $b: expr) => ({
        let bit = $s.mem.read_byte($s.regs.hl()) & (1 << $b);
        $s.regs.set_flag(N_FLAG, false);
        $s.regs.set_flag(H_FLAG, true);
        $s.regs.set_flag(Z_FLAG, bit == 0b0);
        return 3;
    })
}

macro_rules! impl_RES_b_r_x {
    ($s: ident, $b: expr, $x: ident) => (
        $s.regs.$x &= !(1 << $b);
        return 2;
    )
}
macro_rules! impl_RES_b_HLm {
    ($s: ident, $b: expr) => ({
        let hl = $s.regs.hl();
        let v = $s.mem.read_byte(hl) & !(1 << $b);
        $s.mem.write_byte(hl, v);
        return 4;
    })
}

macro_rules! impl_SET_b_r_x {
    ($s: ident, $b: expr, $x: ident) => (
        $s.regs.$x |= 1 << $b;
        return 2;
    )
}
macro_rules! impl_SET_b_HLm {
    ($s: ident, $b: expr) => ({
        let hl = $s.regs.hl();
        let v = $s.mem.read_byte(hl) | (1 << $b);
        $s.mem.write_byte(hl, v);
        return 4;
    })
}

// The CB-prefixed opcodes are implemented in this crate for better clarity in
// the code. Notations used :
// - (X) means the value stored in memory at the X address
#[allow(non_snake_case)]
impl<M> Cpu<M> where M: Memory + MemoryManagementUnit {

    //
    // --- ROTATE ---
    //

    // RLC : rotate left with carry
    pub fn RLC_r_b(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, b); }
    pub fn RLC_r_c(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, c); }
    pub fn RLC_r_d(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, d); }
    pub fn RLC_r_e(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, e); }
    pub fn RLC_r_h(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, h); }
    pub fn RLC_r_l(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, l); }
    pub fn RLC_r_a(&mut self) -> CycleType { impl_rot_reg!(self, alu_rlc, a); }
    pub fn RLC_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        temp_byte = self.alu_rlc(temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }
    // RL : rotate left
    pub fn RL_r_b(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, b); }
    pub fn RL_r_c(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, c); }
    pub fn RL_r_d(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, d); }
    pub fn RL_r_e(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, e); }
    pub fn RL_r_h(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, h); }
    pub fn RL_r_l(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, l); }
    pub fn RL_r_a(&mut self) -> CycleType { impl_rot_reg!(self, alu_rl, a); }
    pub fn RL_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        temp_byte = self.alu_rl(temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }
    // RRC : rotate right with carry
    pub fn RRC_r_b(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, b); }
    pub fn RRC_r_c(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, c); }
    pub fn RRC_r_d(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, d); }
    pub fn RRC_r_e(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, e); }
    pub fn RRC_r_h(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, h); }
    pub fn RRC_r_l(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, l); }
    pub fn RRC_r_a(&mut self) -> CycleType { impl_rot_reg!(self, alu_rrc, a); }
    pub fn RRC_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        temp_byte = self.alu_rrc(temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }
    // RR : rotate right
    pub fn RR_r_b(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, b); }
    pub fn RR_r_c(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, c); }
    pub fn RR_r_d(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, d); }
    pub fn RR_r_e(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, e); }
    pub fn RR_r_h(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, h); }
    pub fn RR_r_l(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, l); }
    pub fn RR_r_a(&mut self) -> CycleType { impl_rot_reg!(self, alu_rr, a); }
    pub fn RR_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        temp_byte = self.alu_rr(temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }

    //
    // --- SHIFT ---
    //

    // SLA : shift left preserving the sign
    pub fn SLA_r_b(&mut self) -> CycleType { sla!(self, self.regs.b); 2 }
    pub fn SLA_r_c(&mut self) -> CycleType { sla!(self, self.regs.c); 2 }
    pub fn SLA_r_d(&mut self) -> CycleType { sla!(self, self.regs.d); 2 }
    pub fn SLA_r_e(&mut self) -> CycleType { sla!(self, self.regs.e); 2 }
    pub fn SLA_r_h(&mut self) -> CycleType { sla!(self, self.regs.h); 2 }
    pub fn SLA_r_l(&mut self) -> CycleType { sla!(self, self.regs.l); 2 }
    pub fn SLA_r_a(&mut self) -> CycleType { sla!(self, self.regs.a); 2 }
    pub fn SLA_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        sla!(self, temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }
    // SRA : shift right preserving the sign
    pub fn SRA_r_b(&mut self) -> CycleType { sra!(self, self.regs.b); 2 }
    pub fn SRA_r_c(&mut self) -> CycleType { sra!(self, self.regs.c); 2 }
    pub fn SRA_r_d(&mut self) -> CycleType { sra!(self, self.regs.d); 2 }
    pub fn SRA_r_e(&mut self) -> CycleType { sra!(self, self.regs.e); 2 }
    pub fn SRA_r_h(&mut self) -> CycleType { sra!(self, self.regs.h); 2 }
    pub fn SRA_r_l(&mut self) -> CycleType { sra!(self, self.regs.l); 2 }
    pub fn SRA_r_a(&mut self) -> CycleType { sra!(self, self.regs.a); 2 }
    pub fn SRA_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        sra!(self, temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }
    // SRL : shift right
    pub fn SRL_r_b(&mut self) -> CycleType { srl!(self, self.regs.b); 2 }
    pub fn SRL_r_c(&mut self) -> CycleType { srl!(self, self.regs.c); 2 }
    pub fn SRL_r_d(&mut self) -> CycleType { srl!(self, self.regs.d); 2 }
    pub fn SRL_r_e(&mut self) -> CycleType { srl!(self, self.regs.e); 2 }
    pub fn SRL_r_h(&mut self) -> CycleType { srl!(self, self.regs.h); 2 }
    pub fn SRL_r_l(&mut self) -> CycleType { srl!(self, self.regs.l); 2 }
    pub fn SRL_r_a(&mut self) -> CycleType { srl!(self, self.regs.a); 2 }
    pub fn SRL_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        srl!(self, temp_byte);
        self.mem.write_byte(hl, temp_byte);
        4
    }

    //
    // --- SWAP ---
    //

    // SWAP_r_X : swap register X's nibbles, reset NHC flags and set Z flag
    pub fn SWAP_r_b(&mut self) -> CycleType { swap!(self, self.regs.b); 2 }
    pub fn SWAP_r_c(&mut self) -> CycleType { swap!(self, self.regs.c); 2 }
    pub fn SWAP_r_d(&mut self) -> CycleType { swap!(self, self.regs.d); 2 }
    pub fn SWAP_r_e(&mut self) -> CycleType { swap!(self, self.regs.e); 2 }
    pub fn SWAP_r_h(&mut self) -> CycleType { swap!(self, self.regs.h); 2 }
    pub fn SWAP_r_l(&mut self) -> CycleType { swap!(self, self.regs.l); 2 }
    pub fn SWAP_r_a(&mut self) -> CycleType { swap!(self, self.regs.a); 2 }
    // SWAP_HLm : same but for (HL)
    pub fn SWAP_HLm(&mut self) -> CycleType {
        let address = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(address);
        swap!(self, temp_byte);
        self.mem.write_byte(address, temp_byte);
        4
    }

    //
    // --- BIT ---
    //

    // BIT b, X : set the Z flag against the byte of index b in register X
    // also set the H flag to 1 and the N flag to 0
    pub fn BIT_0_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, b); }
    pub fn BIT_0_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, c); }
    pub fn BIT_0_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, d); }
    pub fn BIT_0_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, e); }
    pub fn BIT_0_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, h); }
    pub fn BIT_0_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, l); }
    pub fn BIT_0_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 0, a); }

    pub fn BIT_1_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, b); }
    pub fn BIT_1_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, c); }
    pub fn BIT_1_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, d); }
    pub fn BIT_1_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, e); }
    pub fn BIT_1_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, h); }
    pub fn BIT_1_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, l); }
    pub fn BIT_1_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 1, a); }

    pub fn BIT_2_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, b); }
    pub fn BIT_2_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, c); }
    pub fn BIT_2_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, d); }
    pub fn BIT_2_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, e); }
    pub fn BIT_2_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, h); }
    pub fn BIT_2_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, l); }
    pub fn BIT_2_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 2, a); }

    pub fn BIT_3_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, b); }
    pub fn BIT_3_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, c); }
    pub fn BIT_3_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, d); }
    pub fn BIT_3_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, e); }
    pub fn BIT_3_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, h); }
    pub fn BIT_3_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, l); }
    pub fn BIT_3_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 3, a); }

    pub fn BIT_4_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, b); }
    pub fn BIT_4_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, c); }
    pub fn BIT_4_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, d); }
    pub fn BIT_4_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, e); }
    pub fn BIT_4_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, h); }
    pub fn BIT_4_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, l); }
    pub fn BIT_4_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 4, a); }

    pub fn BIT_5_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, b); }
    pub fn BIT_5_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, c); }
    pub fn BIT_5_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, d); }
    pub fn BIT_5_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, e); }
    pub fn BIT_5_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, h); }
    pub fn BIT_5_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, l); }
    pub fn BIT_5_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 5, a); }

    pub fn BIT_6_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, b); }
    pub fn BIT_6_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, c); }
    pub fn BIT_6_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, d); }
    pub fn BIT_6_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, e); }
    pub fn BIT_6_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, h); }
    pub fn BIT_6_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, l); }
    pub fn BIT_6_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 6, a); }

    pub fn BIT_7_r_b(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, b); }
    pub fn BIT_7_r_c(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, c); }
    pub fn BIT_7_r_d(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, d); }
    pub fn BIT_7_r_e(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, e); }
    pub fn BIT_7_r_h(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, h); }
    pub fn BIT_7_r_l(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, l); }
    pub fn BIT_7_r_a(&mut self) -> CycleType { impl_BIT_b_r_x!(self, 7, a); }

    // BIT b, (HL) : set the Z flag against the byte of index b in (HL)
    // also set the H flag to 1 and the N flag to 0
    pub fn BIT_0_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 0); }
    pub fn BIT_1_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 1); }
    pub fn BIT_2_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 2); }
    pub fn BIT_3_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 3); }
    pub fn BIT_4_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 4); }
    pub fn BIT_5_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 5); }
    pub fn BIT_6_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 6); }
    pub fn BIT_7_HLm(&mut self) -> CycleType { impl_BIT_b_HLm!(self, 7); }

    //
    // --- RES ---
    //
    // RES b, X : set to 0 the byte of index b in register X
    pub fn RES_0_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, b); }
    pub fn RES_0_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, c); }
    pub fn RES_0_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, d); }
    pub fn RES_0_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, e); }
    pub fn RES_0_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, h); }
    pub fn RES_0_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, l); }
    pub fn RES_0_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 0, a); }

    pub fn RES_1_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, b); }
    pub fn RES_1_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, c); }
    pub fn RES_1_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, d); }
    pub fn RES_1_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, e); }
    pub fn RES_1_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, h); }
    pub fn RES_1_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, l); }
    pub fn RES_1_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 1, a); }

    pub fn RES_2_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, b); }
    pub fn RES_2_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, c); }
    pub fn RES_2_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, d); }
    pub fn RES_2_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, e); }
    pub fn RES_2_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, h); }
    pub fn RES_2_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, l); }
    pub fn RES_2_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 2, a); }

    pub fn RES_3_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, b); }
    pub fn RES_3_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, c); }
    pub fn RES_3_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, d); }
    pub fn RES_3_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, e); }
    pub fn RES_3_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, h); }
    pub fn RES_3_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, l); }
    pub fn RES_3_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 3, a); }

    pub fn RES_4_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, b); }
    pub fn RES_4_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, c); }
    pub fn RES_4_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, d); }
    pub fn RES_4_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, e); }
    pub fn RES_4_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, h); }
    pub fn RES_4_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, l); }
    pub fn RES_4_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 4, a); }

    pub fn RES_5_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, b); }
    pub fn RES_5_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, c); }
    pub fn RES_5_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, d); }
    pub fn RES_5_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, e); }
    pub fn RES_5_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, h); }
    pub fn RES_5_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, l); }
    pub fn RES_5_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 5, a); }

    pub fn RES_6_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, b); }
    pub fn RES_6_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, c); }
    pub fn RES_6_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, d); }
    pub fn RES_6_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, e); }
    pub fn RES_6_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, h); }
    pub fn RES_6_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, l); }
    pub fn RES_6_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 6, a); }

    pub fn RES_7_r_b(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, b); }
    pub fn RES_7_r_c(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, c); }
    pub fn RES_7_r_d(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, d); }
    pub fn RES_7_r_e(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, e); }
    pub fn RES_7_r_h(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, h); }
    pub fn RES_7_r_l(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, l); }
    pub fn RES_7_r_a(&mut self) -> CycleType { impl_RES_b_r_x!(self, 7, a); }

    // RES b, (HL) : set to 0 the byte of index b in (HL)
    pub fn RES_0_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 0); }
    pub fn RES_1_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 1); }
    pub fn RES_2_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 2); }
    pub fn RES_3_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 3); }
    pub fn RES_4_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 4); }
    pub fn RES_5_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 5); }
    pub fn RES_6_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 6); }
    pub fn RES_7_HLm(&mut self) -> CycleType { impl_RES_b_HLm!(self, 7); }

    //
    // --- SET ---
    //

    // SET b, X : set to 1 the byte of index b in register X
    pub fn SET_0_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, b); }
    pub fn SET_0_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, c); }
    pub fn SET_0_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, d); }
    pub fn SET_0_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, e); }
    pub fn SET_0_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, h); }
    pub fn SET_0_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, l); }
    pub fn SET_0_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 0, a); }

    pub fn SET_1_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, b); }
    pub fn SET_1_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, c); }
    pub fn SET_1_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, d); }
    pub fn SET_1_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, e); }
    pub fn SET_1_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, h); }
    pub fn SET_1_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, l); }
    pub fn SET_1_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 1, a); }

    pub fn SET_2_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, b); }
    pub fn SET_2_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, c); }
    pub fn SET_2_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, d); }
    pub fn SET_2_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, e); }
    pub fn SET_2_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, h); }
    pub fn SET_2_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, l); }
    pub fn SET_2_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 2, a); }

    pub fn SET_3_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, b); }
    pub fn SET_3_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, c); }
    pub fn SET_3_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, d); }
    pub fn SET_3_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, e); }
    pub fn SET_3_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, h); }
    pub fn SET_3_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, l); }
    pub fn SET_3_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 3, a); }

    pub fn SET_4_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, b); }
    pub fn SET_4_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, c); }
    pub fn SET_4_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, d); }
    pub fn SET_4_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, e); }
    pub fn SET_4_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, h); }
    pub fn SET_4_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, l); }
    pub fn SET_4_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 4, a); }

    pub fn SET_5_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, b); }
    pub fn SET_5_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, c); }
    pub fn SET_5_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, d); }
    pub fn SET_5_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, e); }
    pub fn SET_5_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, h); }
    pub fn SET_5_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, l); }
    pub fn SET_5_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 5, a); }

    pub fn SET_6_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, b); }
    pub fn SET_6_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, c); }
    pub fn SET_6_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, d); }
    pub fn SET_6_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, e); }
    pub fn SET_6_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, h); }
    pub fn SET_6_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, l); }
    pub fn SET_6_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 6, a); }

    pub fn SET_7_r_b(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, b); }
    pub fn SET_7_r_c(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, c); }
    pub fn SET_7_r_d(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, d); }
    pub fn SET_7_r_e(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, e); }
    pub fn SET_7_r_h(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, h); }
    pub fn SET_7_r_l(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, l); }
    pub fn SET_7_r_a(&mut self) -> CycleType { impl_SET_b_r_x!(self, 7, a); }

    // SET b, (HL) : set to 1 the byte of index b in (HL)
    pub fn SET_0_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 0); }
    pub fn SET_1_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 1); }
    pub fn SET_2_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 2); }
    pub fn SET_3_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 3); }
    pub fn SET_4_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 4); }
    pub fn SET_5_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 5); }
    pub fn SET_6_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 6); }
    pub fn SET_7_HLm(&mut self) -> CycleType { impl_SET_b_HLm!(self, 7); }
}
