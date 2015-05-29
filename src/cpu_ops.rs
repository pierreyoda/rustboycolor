use super::cpu::*;
use super::memory::Memory;
use super::registers::{Z_FLAG, N_FLAG, H_FLAG, C_FLAG};

// avoid boilerplate for the LD[]_xy functions
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

// The opcodes are implemented in this crate for better clarity in the code.
// Notations used :
// - (X) means the value stored in memory at the X address
#[allow(non_snake_case)]
impl<M> Cpu<M> where M: Memory {

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

    //
    // --- JP ---
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
}
