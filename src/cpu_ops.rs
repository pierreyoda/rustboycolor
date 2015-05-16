use super::cpu::*;
use super::memory::Memory;

// avoid boilerplate for the LD[]_xy functions
// cannot macro the whole function declaration since 'concat_indents!'
// cannot work (yet) for function declarations
macro_rules! impl_LDrr_xy {
    ($s: ident, $x: ident, $y: ident) => (
            $s.regs.$x = $s.regs.$y;
            return 1;
    )
}

macro_rules! name {
    () => ()
}

// The opcodes are implemented in this crate for better clarity in the code.
#[warn(non_snake_case)]
impl<M> Cpu<M> where M: Memory {

    pub fn nop(&mut self) -> CycleType {
        1
    }

    //
    // --- LD ---
    //

    // LDrr_xy : load register y in register x
    pub fn ldrr_bb(&mut self) -> CycleType { impl_LDrr_xy!(self,b,b); }
    pub fn ldrr_bc(&mut self) -> CycleType { impl_LDrr_xy!(self,b,c); }
    pub fn ldrr_bd(&mut self) -> CycleType { impl_LDrr_xy!(self,b,d); }
    pub fn ldrr_be(&mut self) -> CycleType { impl_LDrr_xy!(self,b,e); }
    pub fn ldrr_bh(&mut self) -> CycleType { impl_LDrr_xy!(self,b,h); }
    pub fn ldrr_bl(&mut self) -> CycleType { impl_LDrr_xy!(self,b,l); }
    pub fn ldrr_ba(&mut self) -> CycleType { impl_LDrr_xy!(self,b,a); }

    pub fn ldrr_cb(&mut self) -> CycleType { impl_LDrr_xy!(self,c,b); }
    pub fn ldrr_cc(&mut self) -> CycleType { impl_LDrr_xy!(self,c,c); }
    pub fn ldrr_cd(&mut self) -> CycleType { impl_LDrr_xy!(self,c,d); }
    pub fn ldrr_ce(&mut self) -> CycleType { impl_LDrr_xy!(self,c,e); }
    pub fn ldrr_ch(&mut self) -> CycleType { impl_LDrr_xy!(self,c,h); }
    pub fn ldrr_cl(&mut self) -> CycleType { impl_LDrr_xy!(self,c,l); }
    pub fn ldrr_ca(&mut self) -> CycleType { impl_LDrr_xy!(self,c,a); }

    pub fn ldrr_db(&mut self) -> CycleType { impl_LDrr_xy!(self,d,b); }
    pub fn ldrr_dc(&mut self) -> CycleType { impl_LDrr_xy!(self,d,c); }
    pub fn ldrr_dd(&mut self) -> CycleType { impl_LDrr_xy!(self,d,d); }
    pub fn ldrr_de(&mut self) -> CycleType { impl_LDrr_xy!(self,d,e); }
    pub fn ldrr_dh(&mut self) -> CycleType { impl_LDrr_xy!(self,d,h); }
    pub fn ldrr_dl(&mut self) -> CycleType { impl_LDrr_xy!(self,d,l); }
    pub fn ldrr_da(&mut self) -> CycleType { impl_LDrr_xy!(self,d,a); }

    pub fn ldrr_eb(&mut self) -> CycleType { impl_LDrr_xy!(self,e,b); }
    pub fn ldrr_ec(&mut self) -> CycleType { impl_LDrr_xy!(self,e,c); }
    pub fn ldrr_ed(&mut self) -> CycleType { impl_LDrr_xy!(self,e,d); }
    pub fn ldrr_ee(&mut self) -> CycleType { impl_LDrr_xy!(self,e,e); }
    pub fn ldrr_eh(&mut self) -> CycleType { impl_LDrr_xy!(self,e,h); }
    pub fn ldrr_el(&mut self) -> CycleType { impl_LDrr_xy!(self,e,l); }
    pub fn ldrr_ea(&mut self) -> CycleType { impl_LDrr_xy!(self,e,a); }

    pub fn ldrr_hb(&mut self) -> CycleType { impl_LDrr_xy!(self,h,b); }
    pub fn ldrr_hc(&mut self) -> CycleType { impl_LDrr_xy!(self,h,c); }
    pub fn ldrr_hd(&mut self) -> CycleType { impl_LDrr_xy!(self,h,d); }
    pub fn ldrr_he(&mut self) -> CycleType { impl_LDrr_xy!(self,h,e); }
    pub fn ldrr_hh(&mut self) -> CycleType { impl_LDrr_xy!(self,h,h); }
    pub fn ldrr_hl(&mut self) -> CycleType { impl_LDrr_xy!(self,h,l); }
    pub fn ldrr_ha(&mut self) -> CycleType { impl_LDrr_xy!(self,h,a); }

    pub fn ldrr_lb(&mut self) -> CycleType { impl_LDrr_xy!(self,l,b); }
    pub fn ldrr_lc(&mut self) -> CycleType { impl_LDrr_xy!(self,l,c); }
    pub fn ldrr_ld(&mut self) -> CycleType { impl_LDrr_xy!(self,l,d); }
    pub fn ldrr_le(&mut self) -> CycleType { impl_LDrr_xy!(self,l,e); }
    pub fn ldrr_lh(&mut self) -> CycleType { impl_LDrr_xy!(self,l,h); }
    pub fn ldrr_ll(&mut self) -> CycleType { impl_LDrr_xy!(self,l,l); }
    pub fn ldrr_la(&mut self) -> CycleType { impl_LDrr_xy!(self,l,a); }

    pub fn ldrr_ab(&mut self) -> CycleType { impl_LDrr_xy!(self,a,b); }
    pub fn ldrr_ac(&mut self) -> CycleType { impl_LDrr_xy!(self,a,c); }
    pub fn ldrr_ad(&mut self) -> CycleType { impl_LDrr_xy!(self,a,d); }
    pub fn ldrr_ae(&mut self) -> CycleType { impl_LDrr_xy!(self,a,e); }
    pub fn ldrr_ah(&mut self) -> CycleType { impl_LDrr_xy!(self,a,h); }
    pub fn ldrr_al(&mut self) -> CycleType { impl_LDrr_xy!(self,a,l); }
    pub fn ldrr_aa(&mut self) -> CycleType { impl_LDrr_xy!(self,a,a); }

    // LDrHL_x : load the (HL) value in register x
}
