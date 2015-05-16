use super::cpu::*;
use super::memory::Memory;

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
        $s.regs.$x = $s.mem.read_byte(($s.regs.h as u16) << 8 + $s.regs.l as u16);
        return 2;
    )
}
macro_rules! impl_LD_HLm_r_x {
    ($s: ident, $x: ident) => (
        $s.mem.write_byte(($s.regs.h as u16) << 8 + $s.regs.l as u16,
                          $s.regs.$x);
        return 2;
    )
}

// The opcodes are implemented in this crate for better clarity in the code.
// Notations used :
// - (HL) means the value stored in memory at the HL address
#[allow(non_snake_case)]
impl<M> Cpu<M> where M: Memory {

    pub fn nop(&mut self) -> CycleType {
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
}
