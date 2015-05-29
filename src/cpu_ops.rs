use super::cpu::*;
use super::memory::Memory;
use super::registers::{Z_FLAG, N_FLAG, H_FLAG, C_FLAG};

// --- Implementation macros ---
// avoid boilerplate some instructions functions
// cannot macro the whole function declaration since 'concat_indents!'
// cannot work (yet) for function declarations

macro_rules! impl_LD_rr_xy {
    ($s: ident, $x: ident, $y: ident) => (
        $s.regs.$x = $s.regs.$y;
        return 1;
    )
}

macro_rules! impl_LD_r_HLm_x {
    ($s: ident, $x: ident) => (
        $s.regs.$x = $s.mem.read_byte($s.regs.hl());
        return 2;
    )
}
macro_rules! impl_LD_HLm_r_x {
    ($s: ident, $x: ident) => (
        $s.mem.write_byte($s.regs.hl(), $s.regs.$x);
        return 2;
    )
}

macro_rules! impl_LD_r_n_x {
    ($s: ident, $x: ident) => (
        $s.regs.$x = $s.fetch_byte();
        return 2;
    )
}

// --- Helper macros ---
// TODO : use more methods instead

macro_rules! inc_byte {
    ($s: ident, $b: expr) => ({
        let r = $b.wrapping_add(1);
        $s.regs.set_flag(Z_FLAG, r == 0x0);
        $s.regs.set_flag(N_FLAG, false);
        $s.regs.set_flag(H_FLAG, r & 0x0F == 0x00);
        $b = r;
    })
}
macro_rules! dec_byte {
    ($s: ident, $b: expr) => ({
        let r = $b.wrapping_sub(1);
        $s.regs.set_flag(Z_FLAG, r == 0x0);
        $s.regs.set_flag(N_FLAG, true);
        $s.regs.set_flag(H_FLAG, r & 0x0F == 0x0F);
        $b = r;
    })
}

// The opcodes are implemented in this crate for better clarity in the code.
// Notations used :
// - (X) means the value stored in memory at the X address
#[allow(non_snake_case)]
impl<M> Cpu<M> where M: Memory {

    //
    // --- Control intructions ---
    //

    pub fn nop(&mut self) -> CycleType {
        1
    }

    pub fn stop(&mut self) -> CycleType {
        self.stop = true;
        1
    }

    pub fn halt(&mut self) -> CycleType {
        self.halt = true;
        1
    }

    //
    // --- LD ---
    //

    // LDrr_xy : load register y in register x
    pub fn LD_rr_bb(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,b); }
    pub fn LD_rr_bc(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,c); }
    pub fn LD_rr_bd(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,d); }
    pub fn LD_rr_be(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,e); }
    pub fn LD_rr_bh(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,h); }
    pub fn LD_rr_bl(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,l); }
    pub fn LD_rr_ba(&mut self) -> CycleType { impl_LD_rr_xy!(self,b,a); }

    pub fn LD_rr_cb(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,b); }
    pub fn LD_rr_cc(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,c); }
    pub fn LD_rr_cd(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,d); }
    pub fn LD_rr_ce(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,e); }
    pub fn LD_rr_ch(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,h); }
    pub fn LD_rr_cl(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,l); }
    pub fn LD_rr_ca(&mut self) -> CycleType { impl_LD_rr_xy!(self,c,a); }

    pub fn LD_rr_db(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,b); }
    pub fn LD_rr_dc(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,c); }
    pub fn LD_rr_dd(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,d); }
    pub fn LD_rr_de(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,e); }
    pub fn LD_rr_dh(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,h); }
    pub fn LD_rr_dl(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,l); }
    pub fn LD_rr_da(&mut self) -> CycleType { impl_LD_rr_xy!(self,d,a); }

    pub fn LD_rr_eb(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,b); }
    pub fn LD_rr_ec(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,c); }
    pub fn LD_rr_ed(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,d); }
    pub fn LD_rr_ee(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,e); }
    pub fn LD_rr_eh(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,h); }
    pub fn LD_rr_el(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,l); }
    pub fn LD_rr_ea(&mut self) -> CycleType { impl_LD_rr_xy!(self,e,a); }

    pub fn LD_rr_hb(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,b); }
    pub fn LD_rr_hc(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,c); }
    pub fn LD_rr_hd(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,d); }
    pub fn LD_rr_he(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,e); }
    pub fn LD_rr_hh(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,h); }
    pub fn LD_rr_hl(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,l); }
    pub fn LD_rr_ha(&mut self) -> CycleType { impl_LD_rr_xy!(self,h,a); }

    pub fn LD_rr_lb(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,b); }
    pub fn LD_rr_lc(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,c); }
    pub fn LD_rr_ld(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,d); }
    pub fn LD_rr_le(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,e); }
    pub fn LD_rr_lh(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,h); }
    pub fn LD_rr_ll(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,l); }
    pub fn LD_rr_la(&mut self) -> CycleType { impl_LD_rr_xy!(self,l,a); }

    pub fn LD_rr_ab(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,b); }
    pub fn LD_rr_ac(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,c); }
    pub fn LD_rr_ad(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,d); }
    pub fn LD_rr_ae(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,e); }
    pub fn LD_rr_ah(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,h); }
    pub fn LD_rr_al(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,l); }
    pub fn LD_rr_aa(&mut self) -> CycleType { impl_LD_rr_xy!(self,a,a); }

    // LD_r_HLm_x : load the (HL) value in register x
    pub fn LD_r_HLm_b(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,b); }
    pub fn LD_r_HLm_c(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,c); }
    pub fn LD_r_HLm_d(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,d); }
    pub fn LD_r_HLm_e(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,e); }
    pub fn LD_r_HLm_h(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,h); }
    pub fn LD_r_HLm_l(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,l); }
    pub fn LD_r_HLm_a(&mut self) -> CycleType { impl_LD_r_HLm_x!(self,a); }

    // LD_HLm_r_x : load the register x value in (HL)
    pub fn LD_HLm_r_b(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,b); }
    pub fn LD_HLm_r_c(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,c); }
    pub fn LD_HLm_r_d(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,d); }
    pub fn LD_HLm_r_e(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,e); }
    pub fn LD_HLm_r_h(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,h); }
    pub fn LD_HLm_r_l(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,l); }
    pub fn LD_HLm_r_a(&mut self) -> CycleType { impl_LD_HLm_r_x!(self,a); }

    // LD_r_n_x : load immediate byte into register x
    pub fn LD_r_n_b(&mut self) -> CycleType { impl_LD_r_n_x!(self,b); }
    pub fn LD_r_n_c(&mut self) -> CycleType { impl_LD_r_n_x!(self,c); }
    pub fn LD_r_n_d(&mut self) -> CycleType { impl_LD_r_n_x!(self,d); }
    pub fn LD_r_n_e(&mut self) -> CycleType { impl_LD_r_n_x!(self,e); }
    pub fn LD_r_n_h(&mut self) -> CycleType { impl_LD_r_n_x!(self,h); }
    pub fn LD_r_n_l(&mut self) -> CycleType { impl_LD_r_n_x!(self,l); }
    pub fn LD_r_n_a(&mut self) -> CycleType { impl_LD_r_n_x!(self,a); }

    // LD_HLm_n : load immediate byte into (HL)
    pub fn LD_HLm_n(&mut self) -> CycleType {
        let n = self.fetch_byte();
        self.mem.write_byte(self.regs.hl(), n);
        3
    }

    // LD_XYm_A : load A into (rXrY)
    pub fn LD_BCm_A(&mut self) -> CycleType {
        self.mem.write_byte(self.regs.bc(), self.regs.a);
        2
    }
    pub fn LD_DEm_A(&mut self) -> CycleType {
        self.mem.write_byte(self.regs.de(), self.regs.a);
        2
    }

    // LD_A_XYm : load (rXrY) into A
    pub fn LD_A_BCm(&mut self) -> CycleType {
        self.regs.a = self.mem.read_byte(self.regs.bc());
        2
    }
    pub fn LD_A_DEm(&mut self) -> CycleType {
        self.regs.a = self.mem.read_byte(self.regs.de());
        2
    }

    // LD_XY_nn : load immediate word (16 bits) into XY
    pub fn LD_BC_nn(&mut self) -> CycleType {
        self.regs.c = self.fetch_byte();
        self.regs.b = self.fetch_byte();
        3
    }
    pub fn LD_DE_nn(&mut self) -> CycleType {
        self.regs.e = self.fetch_byte();
        self.regs.d = self.fetch_byte();
        3
    }
    pub fn LD_HL_nn(&mut self) -> CycleType {
        self.regs.l = self.fetch_byte();
        self.regs.h = self.fetch_byte();
        3
    }
    pub fn LD_SP_nn(&mut self) -> CycleType {
        self.regs.sp = self.fetch_word();
        3
    }

    // LD_NNm_A / LD_A_NNm : load A into (nn) / load (nn) into A
    pub fn LD_NNm_A(&mut self) -> CycleType {
        let nn = self.fetch_word();
        self.mem.write_byte(nn, self.regs.a);
        4
    }
    pub fn LD_A_NNm(&mut self) -> CycleType {
        let nn = self.fetch_word();
        self.regs.a = self.mem.read_byte(nn);
        4
    }

    // LDI_HLm_A / LDD_HLm_A : load A into (HL) and increment/decrement HL
    pub fn LDI_HLm_A(&mut self) -> CycleType {
        let hl = self.regs.hl();
        self.mem.write_byte(hl, self.regs.a);
        self.regs.set_hl(hl+1);
        2
    }
    pub fn LDD_HLm_A(&mut self) -> CycleType {
        let hl = self.regs.hl();
        self.mem.write_byte(hl, self.regs.a);
        self.regs.set_hl(hl-1);
        2
    }

    // LDI_A_HLm / LDD_A_HLm : load (HL) into A and increment/decrement HL
    pub fn LDI_A_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        self.regs.a = self.mem.read_byte(hl);
        self.regs.set_hl(hl+1);
        2
    }
    pub fn LDD_A_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        self.regs.a = self.mem.read_byte(hl);
        self.regs.set_hl(hl-1);
        2
    }

    // LDH_n_A : load A into (0xFF00 + offset = 8-bit immediate)
    pub fn LDH_n_A(&mut self) -> CycleType {
        let n = self.fetch_byte();
        self.mem.write_byte(0xFF00 + n as u16, self.regs.a);
        3
    }
    // LDH_A_n : load (0xFF00 + offset = 8-bit immediate) into A
    pub fn LDH_A_n(&mut self) -> CycleType {
        let n = self.fetch_byte();
        self.regs.a = self.mem.read_byte(0xFF00 + n as u16);
        3
    }

    // LDH_C_A : load A into (0xFF00 + offset = C)
    pub fn LDH_C_A(&mut self) -> CycleType {
        self.mem.write_byte(0xFF00 + self.regs.c as u16, self.regs.a);
        2
    }
    // LDH_A_C : load (0xFF00 + offset = C) into A
    pub fn LDH_A_C(&mut self) -> CycleType {
        self.regs.a = self.mem.read_byte(0xFF00 + self.regs.c as u16);
        2
    }

    // LDHL_SP_n : add signed 8-bit immediate to SP and save the result in HL
    pub fn LDHL_SP_n(&mut self) -> CycleType {
        let v = (self.fetch_byte() as i8 as i16 as u16) + self.regs.sp;
        self.regs.set_hl(v);
        3
    }

    // LD_SP_HL : load HL into SP
    pub fn LD_SP_HL(&mut self) -> CycleType {
        self.regs.sp = self.regs.hl();
        2
    }

    // LD_NNm_SP : load SP into (NN) where NN = immediate word
    pub fn LD_NNm_SP(&mut self) -> CycleType {
        let nn = self.fetch_word();
        self.mem.write_word(nn, self.regs.sp);
        5
    }

    //
    // --- Arithmetic Operations ---
    //

    // INC_r_x / DEC_r_x : increment/decrement register X
    pub fn INC_r_b(&mut self) -> CycleType { inc_byte!(self, self.regs.b); 1 }
    pub fn INC_r_c(&mut self) -> CycleType { inc_byte!(self, self.regs.c); 1 }
    pub fn INC_r_d(&mut self) -> CycleType { inc_byte!(self, self.regs.d); 1 }
    pub fn INC_r_e(&mut self) -> CycleType { inc_byte!(self, self.regs.e); 1 }
    pub fn INC_r_h(&mut self) -> CycleType { inc_byte!(self, self.regs.h); 1 }
    pub fn INC_r_l(&mut self) -> CycleType { inc_byte!(self, self.regs.l); 1 }
    pub fn INC_r_a(&mut self) -> CycleType { inc_byte!(self, self.regs.a); 1 }

    pub fn DEC_r_b(&mut self) -> CycleType { dec_byte!(self, self.regs.b); 1 }
    pub fn DEC_r_c(&mut self) -> CycleType { dec_byte!(self, self.regs.c); 1 }
    pub fn DEC_r_d(&mut self) -> CycleType { dec_byte!(self, self.regs.d); 1 }
    pub fn DEC_r_e(&mut self) -> CycleType { dec_byte!(self, self.regs.e); 1 }
    pub fn DEC_r_h(&mut self) -> CycleType { dec_byte!(self, self.regs.h); 1 }
    pub fn DEC_r_l(&mut self) -> CycleType { dec_byte!(self, self.regs.l); 1 }
    pub fn DEC_r_a(&mut self) -> CycleType { dec_byte!(self, self.regs.a); 1 }

    // INC_HLm / DEC_HLm : increment/decrement (HL)
    pub fn INC_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        inc_byte!(self, temp_byte);
        self.mem.write_byte(hl, temp_byte);
        3
    }
    pub fn DEC_HLm(&mut self) -> CycleType {
        let hl = self.regs.hl();
        let mut temp_byte = self.mem.read_byte(hl);
        dec_byte!(self, temp_byte);
        self.mem.write_byte(hl, temp_byte);
        3
    }

    // INC_XY / INC_XY : increment/decrement XY
    pub fn INC_BC(&mut self) -> CycleType {
        let v = self.regs.bc().wrapping_add(1); self.regs.set_bc(v); 2
    }
    pub fn INC_DE(&mut self) -> CycleType {
        let v = self.regs.de().wrapping_add(1); self.regs.set_de(v); 2
    }
    pub fn INC_HL(&mut self) -> CycleType {
        let v = self.regs.hl().wrapping_add(1); self.regs.set_hl(v); 2
    }
    pub fn INC_SP(&mut self) -> CycleType {
        self.regs.sp = self.regs.sp.wrapping_add(1); 2
    }

    pub fn DEC_BC(&mut self) -> CycleType {
        let v = self.regs.bc().wrapping_sub(1); self.regs.set_bc(v); 2
    }
    pub fn DEC_DE(&mut self) -> CycleType {
        let v = self.regs.de().wrapping_sub(1); self.regs.set_de(v); 2
    }
    pub fn DEC_HL(&mut self) -> CycleType {
        let v = self.regs.hl().wrapping_sub(1); self.regs.set_hl(v); 2
    }
    pub fn DEC_SP(&mut self) -> CycleType {
        self.regs.sp = self.regs.sp.wrapping_sub(1); 2
    }

    //
    // --- Jumps / calls ---
    //

    // JP_nn : absolute jump to 16-bit address
    pub fn JP_nn(&mut self) -> CycleType {
        self.regs.pc = self.fetch_word();
        4
    }
    // JP_NZ_nn : absolute jump to 16-bit address if the zero flag is not set
    pub fn JP_NZ_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if !self.regs.flag(Z_FLAG) { self.regs.pc = nn; 4 } else { 3 }
    }
    // JP_NC_nn : absolute jump to 16-bit address if the carry flag is not set
    pub fn JP_NC_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if !self.regs.flag(C_FLAG) { self.regs.pc = nn; 4 } else { 3 }
    }
    // JP_NZ_nn : absolute jump to 16-bit address if the zero flag is set
    pub fn JP_Z_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if self.regs.flag(Z_FLAG) { self.regs.pc = nn; 4 } else { 3 }
    }
    // JP_NC_nn : absolute jump to 16-bit address if the carry flag is set
    pub fn JP_C_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if self.regs.flag(C_FLAG) { self.regs.pc = nn; 4 } else { 3 }
    }
    // JP_HLm : jump to (HL)
    pub fn JP_HLm(&mut self) -> CycleType {
        self.regs.pc = self.regs.hl();
        1
    }

    // JR_n : relative jump by signed immediate byte
    pub fn JR_n(&mut self) -> CycleType {
        let b = self.fetch_byte();
        self.cpu_jr(b);
        3
    }

    // JR_Z_n : relative jump by signed immediate byte if the zero flag is set
    pub fn JR_Z_n(&mut self) -> CycleType {
        let b = self.fetch_byte();
        if self.regs.flag(Z_FLAG) { self.cpu_jr(b); 3 } else { 2 }
    }
    // JR_NZ_n : relative jump by signed immediate byte if the zero flag is not set
    pub fn JR_NZ_n(&mut self) -> CycleType {
        let b = self.fetch_byte();
        if !self.regs.flag(Z_FLAG) { self.cpu_jr(b); 3 } else { 2 }
    }

    // JR_C_n : relative jump by signed immediate byte if the carry flag is set
    pub fn JR_C_n(&mut self) -> CycleType {
        let b = self.fetch_byte();
        if self.regs.flag(C_FLAG) { self.cpu_jr(b); 3 } else { 2 }
    }
    // JR_NC_n : relative jump by signed immediate byte if the carry flag is not set
    pub fn JR_NC_n(&mut self) -> CycleType {
        let b = self.fetch_byte();
        if !self.regs.flag(C_FLAG) { self.cpu_jr(b); 3 } else { 2 }
    }

    // CALL_nn : call routine at 16-bit address
    pub fn CALL_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        self.cpu_call(nn);
        6
    }

    // CALL_Z_nn : call routine at 16-bit address if the zero flag is set
    pub fn CALL_Z_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if self.regs.flag(Z_FLAG) { self.cpu_call(nn); 6 } else { 3 }
    }
    // CALL_NZ_nn : call routine at 16-bit address if the zero flag is not set
    pub fn CALL_NZ_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if !self.regs.flag(Z_FLAG) { self.cpu_call(nn); 6 } else { 3 }
    }

    // CALL_C_nn : call routine at 16-bit address if the carry flag is set
    pub fn CALL_C_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if self.regs.flag(C_FLAG) { self.cpu_call(nn); 6 } else { 3 }
    }
    // CALL_NC_nn : call routine at 16-bit address if the carry flag is not set
    pub fn CALL_NC_nn(&mut self) -> CycleType {
        let nn = self.fetch_word();
        if !self.regs.flag(C_FLAG) { self.cpu_call(nn); 6 } else { 3 }
    }

    // RET : return to calling routine
    pub fn RET(&mut self) -> CycleType {
        self.regs.pc = self.stack_pop();
        4
    }
    // RETI : enable interrupts and return to calling routine
    pub fn RETI(&mut self) -> CycleType {
        self.regs.pc = self.stack_pop();
        self.ime = true;
        4
    }

    // RET_Z : return if the zero flag is set
    pub fn RET_Z(&mut self) -> CycleType {
        if self.regs.flag(Z_FLAG) { self.regs.pc = self.stack_pop(); 5 } else { 2 }
    }
    // RET_NZ : return if the zero flag is not set
    pub fn RET_NZ(&mut self) -> CycleType {
        if !self.regs.flag(Z_FLAG) { self.regs.pc = self.stack_pop(); 5 } else { 2 }
    }

    // RET_C : return if the carry flag is set
    pub fn RET_C(&mut self) -> CycleType {
        if self.regs.flag(C_FLAG) { self.regs.pc = self.stack_pop(); 5 } else { 2 }
    }
    // RET_NC : return if the carry flag is not set
    pub fn RET_NC(&mut self) -> CycleType {
        if !self.regs.flag(C_FLAG) { self.regs.pc = self.stack_pop(); 5 } else { 2 }
    }

    // RST_xxH : call routine at address 0x00XX
    pub fn RST_00H(&mut self) -> CycleType { self.cpu_jr(0x0000); 4 }
    pub fn RST_08H(&mut self) -> CycleType { self.cpu_jr(0x0008); 4 }
    pub fn RST_10H(&mut self) -> CycleType { self.cpu_jr(0x0010); 4 }
    pub fn RST_18H(&mut self) -> CycleType { self.cpu_jr(0x0018); 4 }
    pub fn RST_20H(&mut self) -> CycleType { self.cpu_jr(0x0020); 4 }
    pub fn RST_28H(&mut self) -> CycleType { self.cpu_jr(0x0028); 4 }
    pub fn RST_30H(&mut self) -> CycleType { self.cpu_jr(0x0030); 4 }
    pub fn RST_38H(&mut self) -> CycleType { self.cpu_jr(0x0038); 4 }
}
