/// Crate emulating the behavior of the Sharp LR35902 processor powering the
/// Game Boy (Color).

use super::memory::Memory;
use super::registers::Registers;

/// The type used to count the CPU's machine cycles.
pub type CycleType = u64;

/// The structure holding and emulating the CPU state.
pub struct Cpu<M> {
    /// The number of CPU cycles spent since the start of the emulation.
    cycles: CycleType,
    pub halt: bool,
    pub stop: bool,
    /// The CPU's registers.
    pub regs: Registers,
    /// The memory on which the CPU operates.
    pub mem: M,
    /// The opcode currently executed.
    opcode: u8,
    /// The dispatching array used for instruction decoding.
    dispatch_array: [CpuInstruction<M>; 256],
    /// The dispatching array used for decoding the CB-prefixed additional
    /// instructions.
    cb_dispatch_array: [CpuInstruction<M>; 256],
}

impl<M> Cpu<M> where M: Memory {
    /// Return a new, initialized Cpu instance operating on the given 'Memory'.
    pub fn new(mem: M) -> Cpu<M> {
        Cpu {
            cycles: 0,
            halt: false,
            stop: false,
            regs: Registers::new(),
            mem: mem,
            opcode: 0x0,
            dispatch_array: dispatch_array(),
            cb_dispatch_array: cb_dispatch_array(),
        }
    }

    /// Get an immutable reference to the Registers state.
    pub fn registers(&self) -> &Registers {
        &self.regs
    }

    /// Fetch, decode and execute a new instruction.
    pub fn step(&mut self) {
        self.opcode = self.mem.read_byte(self.regs.pc);
        self.regs.pc += 1;

        self.cycles += self.dispatch_array[self.opcode as usize](self);
    }

    /// Called when encountering the '0xCB' prefix.
    pub fn call_cb(&mut self) -> CycleType {
        self.opcode = self.mem.read_byte(self.regs.pc);
        self.regs.pc += 1;
        // TO CHECK : cycles overhead of 'CB' prefix = 0 ?
        self.cb_dispatch_array[self.opcode as usize](self)
    }

    /// Called when an unknown opcode is encountered.
    /// TODO improved behavior
    pub fn opcode_unknown(&mut self) -> CycleType {
        println!("CPU : unknown opcode 0x{:0>2X} ; stopping",
                 self.opcode);
        self.stop = true;
        0
    }
    /// Called when an unknown CB-prefixed opcode is encountered.
    /// TODO improved behavior
    pub fn cb_opcode_unknown(&mut self) -> CycleType {
        println!("CPU : unknown opcode 0xCB{:0>2X} ; stopping",
                 self.opcode);
        self.stop = true;
        0
    }
}

/// The type of the methods used to execute a CPU instruction.
/// The return value is the number of machine cycles spent.
type CpuInstruction<M> = fn(&mut Cpu<M>) -> CycleType;

macro_rules! cpu_instruction {
    ($cpu_method: ident) => (Cpu::<M>::$cpu_method)
}

/// Return the dispatching array for the CPU to use, i.e. the array where
/// the element of index I corresponds to the 'CpuInstruction<M>' associated
/// to the opcode the hexadecimal value of I.
/// The downside is that all the instructions must be public.
/// Could also work for a Disassembler.
/// Main sources for the opcodes :
/// http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
/// http://imrannazar.com/Gameboy-Z80-Opcode-Map
fn dispatch_array<M: Memory>() -> [CpuInstruction<M>; 256] {
    // the default value is the method opcode_unknown
    let mut dispatch_array = [Cpu::<M>::opcode_unknown as CpuInstruction<M>; 256];

    dispatch_array[0x00] = cpu_instruction!(nop);
    dispatch_array[0x01] = cpu_instruction!(LD_BC_nn);
    dispatch_array[0x02] = cpu_instruction!(LD_BCm_A);
    dispatch_array[0x06] = cpu_instruction!(LD_r_n_b);
    dispatch_array[0x08] = cpu_instruction!(LD_NNm_SP);
    dispatch_array[0x0A] = cpu_instruction!(LD_A_BCm);
    dispatch_array[0x0E] = cpu_instruction!(LD_r_n_c);

    dispatch_array[0x10] = cpu_instruction!(stop);
    dispatch_array[0x11] = cpu_instruction!(LD_DE_nn);
    dispatch_array[0x12] = cpu_instruction!(LD_DEm_A);
    dispatch_array[0x16] = cpu_instruction!(LD_r_n_d);
    dispatch_array[0x1A] = cpu_instruction!(LD_A_DEm);
    dispatch_array[0x1E] = cpu_instruction!(LD_r_n_e);

    dispatch_array[0x21] = cpu_instruction!(LD_HL_nn);
    dispatch_array[0x22] = cpu_instruction!(LDI_HLm_A);
    dispatch_array[0x26] = cpu_instruction!(LD_r_n_h);
    dispatch_array[0x2A] = cpu_instruction!(LDI_A_HLm);
    dispatch_array[0x2E] = cpu_instruction!(LD_r_n_l);

    dispatch_array[0x31] = cpu_instruction!(LD_SP_nn);
    dispatch_array[0x32] = cpu_instruction!(LDD_HLm_A);
    dispatch_array[0x36] = cpu_instruction!(LD_HLm_n);
    dispatch_array[0x3A] = cpu_instruction!(LDD_A_HLm);
    dispatch_array[0x3E] = cpu_instruction!(LD_r_n_a);

    dispatch_array[0x40] = cpu_instruction!(LD_rr_bb);
    dispatch_array[0x41] = cpu_instruction!(LD_rr_bc);
    dispatch_array[0x42] = cpu_instruction!(LD_rr_bd);
    dispatch_array[0x43] = cpu_instruction!(LD_rr_be);
    dispatch_array[0x44] = cpu_instruction!(LD_rr_bh);
    dispatch_array[0x45] = cpu_instruction!(LD_rr_bl);
    dispatch_array[0x46] = cpu_instruction!(LD_r_HLm_b);
    dispatch_array[0x47] = cpu_instruction!(LD_rr_ba);
    dispatch_array[0x48] = cpu_instruction!(LD_rr_cb);
    dispatch_array[0x49] = cpu_instruction!(LD_rr_cc);
    dispatch_array[0x4A] = cpu_instruction!(LD_rr_cd);
    dispatch_array[0x4B] = cpu_instruction!(LD_rr_ce);
    dispatch_array[0x4C] = cpu_instruction!(LD_rr_ch);
    dispatch_array[0x4D] = cpu_instruction!(LD_rr_cl);
    dispatch_array[0x4E] = cpu_instruction!(LD_r_HLm_c);
    dispatch_array[0x4F] = cpu_instruction!(LD_rr_ca);

    dispatch_array[0x50] = cpu_instruction!(LD_rr_db);
    dispatch_array[0x51] = cpu_instruction!(LD_rr_dc);
    dispatch_array[0x52] = cpu_instruction!(LD_rr_dd);
    dispatch_array[0x53] = cpu_instruction!(LD_rr_de);
    dispatch_array[0x54] = cpu_instruction!(LD_rr_dh);
    dispatch_array[0x55] = cpu_instruction!(LD_rr_dl);
    dispatch_array[0x56] = cpu_instruction!(LD_r_HLm_d);
    dispatch_array[0x57] = cpu_instruction!(LD_rr_da);
    dispatch_array[0x58] = cpu_instruction!(LD_rr_eb);
    dispatch_array[0x59] = cpu_instruction!(LD_rr_ec);
    dispatch_array[0x5A] = cpu_instruction!(LD_rr_ed);
    dispatch_array[0x5B] = cpu_instruction!(LD_rr_ee);
    dispatch_array[0x5C] = cpu_instruction!(LD_rr_eh);
    dispatch_array[0x5D] = cpu_instruction!(LD_rr_el);
    dispatch_array[0x5E] = cpu_instruction!(LD_r_HLm_e);
    dispatch_array[0x5F] = cpu_instruction!(LD_rr_ea);

    dispatch_array[0x60] = cpu_instruction!(LD_rr_hb);
    dispatch_array[0x61] = cpu_instruction!(LD_rr_hc);
    dispatch_array[0x62] = cpu_instruction!(LD_rr_hd);
    dispatch_array[0x63] = cpu_instruction!(LD_rr_he);
    dispatch_array[0x64] = cpu_instruction!(LD_rr_hh);
    dispatch_array[0x65] = cpu_instruction!(LD_rr_hl);
    dispatch_array[0x66] = cpu_instruction!(LD_r_HLm_h);
    dispatch_array[0x67] = cpu_instruction!(LD_rr_ha);
    dispatch_array[0x68] = cpu_instruction!(LD_rr_lb);
    dispatch_array[0x69] = cpu_instruction!(LD_rr_lc);
    dispatch_array[0x6A] = cpu_instruction!(LD_rr_ld);
    dispatch_array[0x6B] = cpu_instruction!(LD_rr_le);
    dispatch_array[0x6C] = cpu_instruction!(LD_rr_lh);
    dispatch_array[0x6D] = cpu_instruction!(LD_rr_ll);
    dispatch_array[0x6E] = cpu_instruction!(LD_r_HLm_l);
    dispatch_array[0x6F] = cpu_instruction!(LD_rr_la);

    dispatch_array[0x70] = cpu_instruction!(LD_HLm_r_b);
    dispatch_array[0x71] = cpu_instruction!(LD_HLm_r_c);
    dispatch_array[0x72] = cpu_instruction!(LD_HLm_r_d);
    dispatch_array[0x73] = cpu_instruction!(LD_HLm_r_e);
    dispatch_array[0x74] = cpu_instruction!(LD_HLm_r_h);
    dispatch_array[0x75] = cpu_instruction!(LD_HLm_r_l);
    dispatch_array[0x76] = cpu_instruction!(halt);
    dispatch_array[0x77] = cpu_instruction!(LD_HLm_r_a);
    dispatch_array[0x78] = cpu_instruction!(LD_rr_ab);
    dispatch_array[0x79] = cpu_instruction!(LD_rr_ac);
    dispatch_array[0x7A] = cpu_instruction!(LD_rr_ad);
    dispatch_array[0x7B] = cpu_instruction!(LD_rr_ae);
    dispatch_array[0x7C] = cpu_instruction!(LD_rr_ah);
    dispatch_array[0x7D] = cpu_instruction!(LD_rr_al);
    dispatch_array[0x7E] = cpu_instruction!(LD_r_HLm_a);
    dispatch_array[0x7F] = cpu_instruction!(LD_rr_aa);

    dispatch_array[0xCB] = cpu_instruction!(call_cb);

    dispatch_array[0xE0] = cpu_instruction!(LDH_n_A);
    dispatch_array[0xE2] = cpu_instruction!(LDH_C_A);
    dispatch_array[0xEA] = cpu_instruction!(LD_NNm_A);

    dispatch_array[0xF0] = cpu_instruction!(LDH_A_n);
    dispatch_array[0xF2] = cpu_instruction!(LDH_A_C);
    dispatch_array[0xF8] = cpu_instruction!(LDHL_SP_n);
    dispatch_array[0xF9] = cpu_instruction!(LD_SP_HL);
    dispatch_array[0xFA] = cpu_instruction!(LD_A_NNm);

    dispatch_array
}

/// Same as 'dispatch_array()' but for the additional, CB-prefixed instructions.
fn cb_dispatch_array<M: Memory>() -> [CpuInstruction<M>; 256] {
    // the default value is the method cb_opcode_unknown
    let mut cb_dispatch_array = [Cpu::<M>::cb_opcode_unknown
        as CpuInstruction<M>; 256];

    cb_dispatch_array[0x30] = cpu_instruction!(SWAP_r_b);
    cb_dispatch_array[0x31] = cpu_instruction!(SWAP_r_c);
    cb_dispatch_array[0x32] = cpu_instruction!(SWAP_r_d);
    cb_dispatch_array[0x33] = cpu_instruction!(SWAP_r_e);
    cb_dispatch_array[0x34] = cpu_instruction!(SWAP_r_h);
    cb_dispatch_array[0x35] = cpu_instruction!(SWAP_r_l);
    cb_dispatch_array[0x36] = cpu_instruction!(SWAP_HLm);
    cb_dispatch_array[0x37] = cpu_instruction!(SWAP_r_a);

    cb_dispatch_array[0x40] = cpu_instruction!(BIT_0_r_b);
    cb_dispatch_array[0x41] = cpu_instruction!(BIT_0_r_c);
    cb_dispatch_array[0x42] = cpu_instruction!(BIT_0_r_d);
    cb_dispatch_array[0x43] = cpu_instruction!(BIT_0_r_e);
    cb_dispatch_array[0x44] = cpu_instruction!(BIT_0_r_h);
    cb_dispatch_array[0x45] = cpu_instruction!(BIT_0_r_l);
    cb_dispatch_array[0x46] = cpu_instruction!(BIT_0_HLm);
    cb_dispatch_array[0x47] = cpu_instruction!(BIT_0_r_a);
    cb_dispatch_array[0x48] = cpu_instruction!(BIT_1_r_b);
    cb_dispatch_array[0x49] = cpu_instruction!(BIT_1_r_c);
    cb_dispatch_array[0x4A] = cpu_instruction!(BIT_1_r_d);
    cb_dispatch_array[0x4B] = cpu_instruction!(BIT_1_r_e);
    cb_dispatch_array[0x4C] = cpu_instruction!(BIT_1_r_h);
    cb_dispatch_array[0x4D] = cpu_instruction!(BIT_1_r_l);
    cb_dispatch_array[0x4E] = cpu_instruction!(BIT_1_HLm);
    cb_dispatch_array[0x4F] = cpu_instruction!(BIT_1_r_a);

    cb_dispatch_array[0x50] = cpu_instruction!(BIT_2_r_b);
    cb_dispatch_array[0x51] = cpu_instruction!(BIT_2_r_c);
    cb_dispatch_array[0x52] = cpu_instruction!(BIT_2_r_d);
    cb_dispatch_array[0x53] = cpu_instruction!(BIT_2_r_e);
    cb_dispatch_array[0x54] = cpu_instruction!(BIT_2_r_h);
    cb_dispatch_array[0x55] = cpu_instruction!(BIT_2_r_l);
    cb_dispatch_array[0x56] = cpu_instruction!(BIT_2_HLm);
    cb_dispatch_array[0x57] = cpu_instruction!(BIT_2_r_a);
    cb_dispatch_array[0x58] = cpu_instruction!(BIT_3_r_b);
    cb_dispatch_array[0x59] = cpu_instruction!(BIT_3_r_c);
    cb_dispatch_array[0x5A] = cpu_instruction!(BIT_3_r_d);
    cb_dispatch_array[0x5B] = cpu_instruction!(BIT_3_r_e);
    cb_dispatch_array[0x5C] = cpu_instruction!(BIT_3_r_h);
    cb_dispatch_array[0x5D] = cpu_instruction!(BIT_3_r_l);
    cb_dispatch_array[0x5E] = cpu_instruction!(BIT_3_HLm);
    cb_dispatch_array[0x5F] = cpu_instruction!(BIT_3_r_a);

    cb_dispatch_array[0x60] = cpu_instruction!(BIT_4_r_b);
    cb_dispatch_array[0x61] = cpu_instruction!(BIT_4_r_c);
    cb_dispatch_array[0x62] = cpu_instruction!(BIT_4_r_d);
    cb_dispatch_array[0x63] = cpu_instruction!(BIT_4_r_e);
    cb_dispatch_array[0x64] = cpu_instruction!(BIT_4_r_h);
    cb_dispatch_array[0x65] = cpu_instruction!(BIT_4_r_l);
    cb_dispatch_array[0x66] = cpu_instruction!(BIT_4_HLm);
    cb_dispatch_array[0x67] = cpu_instruction!(BIT_4_r_a);
    cb_dispatch_array[0x68] = cpu_instruction!(BIT_5_r_b);
    cb_dispatch_array[0x69] = cpu_instruction!(BIT_5_r_c);
    cb_dispatch_array[0x6A] = cpu_instruction!(BIT_5_r_d);
    cb_dispatch_array[0x6B] = cpu_instruction!(BIT_5_r_e);
    cb_dispatch_array[0x6C] = cpu_instruction!(BIT_5_r_h);
    cb_dispatch_array[0x6D] = cpu_instruction!(BIT_5_r_l);
    cb_dispatch_array[0x6E] = cpu_instruction!(BIT_5_HLm);
    cb_dispatch_array[0x6F] = cpu_instruction!(BIT_5_r_a);

    cb_dispatch_array[0x70] = cpu_instruction!(BIT_6_r_b);
    cb_dispatch_array[0x71] = cpu_instruction!(BIT_6_r_c);
    cb_dispatch_array[0x72] = cpu_instruction!(BIT_6_r_d);
    cb_dispatch_array[0x73] = cpu_instruction!(BIT_6_r_e);
    cb_dispatch_array[0x74] = cpu_instruction!(BIT_6_r_h);
    cb_dispatch_array[0x75] = cpu_instruction!(BIT_6_r_l);
    cb_dispatch_array[0x76] = cpu_instruction!(BIT_6_HLm);
    cb_dispatch_array[0x77] = cpu_instruction!(BIT_6_r_a);
    cb_dispatch_array[0x78] = cpu_instruction!(BIT_7_r_b);
    cb_dispatch_array[0x79] = cpu_instruction!(BIT_7_r_c);
    cb_dispatch_array[0x7A] = cpu_instruction!(BIT_7_r_d);
    cb_dispatch_array[0x7B] = cpu_instruction!(BIT_7_r_e);
    cb_dispatch_array[0x7C] = cpu_instruction!(BIT_7_r_h);
    cb_dispatch_array[0x7D] = cpu_instruction!(BIT_7_r_l);
    cb_dispatch_array[0x7E] = cpu_instruction!(BIT_7_HLm);
    cb_dispatch_array[0x7F] = cpu_instruction!(BIT_7_r_a);

    cb_dispatch_array[0x80] = cpu_instruction!(RES_0_r_b);
    cb_dispatch_array[0x81] = cpu_instruction!(RES_0_r_c);
    cb_dispatch_array[0x82] = cpu_instruction!(RES_0_r_d);
    cb_dispatch_array[0x83] = cpu_instruction!(RES_0_r_e);
    cb_dispatch_array[0x84] = cpu_instruction!(RES_0_r_h);
    cb_dispatch_array[0x85] = cpu_instruction!(RES_0_r_l);
    cb_dispatch_array[0x86] = cpu_instruction!(RES_0_HLm);
    cb_dispatch_array[0x87] = cpu_instruction!(RES_0_r_a);
    cb_dispatch_array[0x88] = cpu_instruction!(RES_1_r_b);
    cb_dispatch_array[0x89] = cpu_instruction!(RES_1_r_c);
    cb_dispatch_array[0x8A] = cpu_instruction!(RES_1_r_d);
    cb_dispatch_array[0x8B] = cpu_instruction!(RES_1_r_e);
    cb_dispatch_array[0x8C] = cpu_instruction!(RES_1_r_h);
    cb_dispatch_array[0x8D] = cpu_instruction!(RES_1_r_l);
    cb_dispatch_array[0x8E] = cpu_instruction!(RES_1_HLm);
    cb_dispatch_array[0x8F] = cpu_instruction!(RES_1_r_a);

    cb_dispatch_array[0x90] = cpu_instruction!(RES_2_r_b);
    cb_dispatch_array[0x91] = cpu_instruction!(RES_2_r_c);
    cb_dispatch_array[0x92] = cpu_instruction!(RES_2_r_d);
    cb_dispatch_array[0x93] = cpu_instruction!(RES_2_r_e);
    cb_dispatch_array[0x94] = cpu_instruction!(RES_2_r_h);
    cb_dispatch_array[0x95] = cpu_instruction!(RES_2_r_l);
    cb_dispatch_array[0x96] = cpu_instruction!(RES_2_HLm);
    cb_dispatch_array[0x97] = cpu_instruction!(RES_2_r_a);
    cb_dispatch_array[0x98] = cpu_instruction!(RES_3_r_b);
    cb_dispatch_array[0x99] = cpu_instruction!(RES_3_r_c);
    cb_dispatch_array[0x9A] = cpu_instruction!(RES_3_r_d);
    cb_dispatch_array[0x9B] = cpu_instruction!(RES_3_r_e);
    cb_dispatch_array[0x9C] = cpu_instruction!(RES_3_r_h);
    cb_dispatch_array[0x9D] = cpu_instruction!(RES_3_r_l);
    cb_dispatch_array[0x9E] = cpu_instruction!(RES_3_HLm);
    cb_dispatch_array[0x9F] = cpu_instruction!(RES_3_r_a);

    cb_dispatch_array[0xA0] = cpu_instruction!(RES_4_r_b);
    cb_dispatch_array[0xA1] = cpu_instruction!(RES_4_r_c);
    cb_dispatch_array[0xA2] = cpu_instruction!(RES_4_r_d);
    cb_dispatch_array[0xA3] = cpu_instruction!(RES_4_r_e);
    cb_dispatch_array[0xA4] = cpu_instruction!(RES_4_r_h);
    cb_dispatch_array[0xA5] = cpu_instruction!(RES_4_r_l);
    cb_dispatch_array[0xA6] = cpu_instruction!(RES_4_HLm);
    cb_dispatch_array[0xA7] = cpu_instruction!(RES_4_r_a);
    cb_dispatch_array[0xA8] = cpu_instruction!(RES_5_r_b);
    cb_dispatch_array[0xA9] = cpu_instruction!(RES_5_r_c);
    cb_dispatch_array[0xAA] = cpu_instruction!(RES_5_r_d);
    cb_dispatch_array[0xAB] = cpu_instruction!(RES_5_r_e);
    cb_dispatch_array[0xAC] = cpu_instruction!(RES_5_r_h);
    cb_dispatch_array[0xAD] = cpu_instruction!(RES_5_r_l);
    cb_dispatch_array[0xAE] = cpu_instruction!(RES_5_HLm);
    cb_dispatch_array[0xAF] = cpu_instruction!(RES_5_r_a);

    cb_dispatch_array[0xB0] = cpu_instruction!(RES_6_r_b);
    cb_dispatch_array[0xB1] = cpu_instruction!(RES_6_r_c);
    cb_dispatch_array[0xB2] = cpu_instruction!(RES_6_r_d);
    cb_dispatch_array[0xB3] = cpu_instruction!(RES_6_r_e);
    cb_dispatch_array[0xB4] = cpu_instruction!(RES_6_r_h);
    cb_dispatch_array[0xB5] = cpu_instruction!(RES_6_r_l);
    cb_dispatch_array[0xB6] = cpu_instruction!(RES_6_HLm);
    cb_dispatch_array[0xB7] = cpu_instruction!(RES_6_r_a);
    cb_dispatch_array[0xB8] = cpu_instruction!(RES_7_r_b);
    cb_dispatch_array[0xB9] = cpu_instruction!(RES_7_r_c);
    cb_dispatch_array[0xBA] = cpu_instruction!(RES_7_r_d);
    cb_dispatch_array[0xBB] = cpu_instruction!(RES_7_r_e);
    cb_dispatch_array[0xBC] = cpu_instruction!(RES_7_r_h);
    cb_dispatch_array[0xBD] = cpu_instruction!(RES_7_r_l);
    cb_dispatch_array[0xBE] = cpu_instruction!(RES_7_HLm);
    cb_dispatch_array[0xBF] = cpu_instruction!(RES_7_r_a);

    cb_dispatch_array[0xC0] = cpu_instruction!(SET_0_r_b);
    cb_dispatch_array[0xC1] = cpu_instruction!(SET_0_r_c);
    cb_dispatch_array[0xC2] = cpu_instruction!(SET_0_r_d);
    cb_dispatch_array[0xC3] = cpu_instruction!(SET_0_r_e);
    cb_dispatch_array[0xC4] = cpu_instruction!(SET_0_r_h);
    cb_dispatch_array[0xC5] = cpu_instruction!(SET_0_r_l);
    cb_dispatch_array[0xC6] = cpu_instruction!(SET_0_HLm);
    cb_dispatch_array[0xC7] = cpu_instruction!(SET_0_r_a);
    cb_dispatch_array[0xC8] = cpu_instruction!(SET_1_r_b);
    cb_dispatch_array[0xC9] = cpu_instruction!(SET_1_r_c);
    cb_dispatch_array[0xCA] = cpu_instruction!(SET_1_r_d);
    cb_dispatch_array[0xCB] = cpu_instruction!(SET_1_r_e);
    cb_dispatch_array[0xCC] = cpu_instruction!(SET_1_r_h);
    cb_dispatch_array[0xCD] = cpu_instruction!(SET_1_r_l);
    cb_dispatch_array[0xCE] = cpu_instruction!(SET_1_HLm);
    cb_dispatch_array[0xCF] = cpu_instruction!(SET_1_r_a);

    cb_dispatch_array[0xD0] = cpu_instruction!(SET_2_r_b);
    cb_dispatch_array[0xD1] = cpu_instruction!(SET_2_r_c);
    cb_dispatch_array[0xD2] = cpu_instruction!(SET_2_r_d);
    cb_dispatch_array[0xD3] = cpu_instruction!(SET_2_r_e);
    cb_dispatch_array[0xD4] = cpu_instruction!(SET_2_r_h);
    cb_dispatch_array[0xD5] = cpu_instruction!(SET_2_r_l);
    cb_dispatch_array[0xD6] = cpu_instruction!(SET_2_HLm);
    cb_dispatch_array[0xD7] = cpu_instruction!(SET_2_r_a);
    cb_dispatch_array[0xD8] = cpu_instruction!(SET_3_r_b);
    cb_dispatch_array[0xD9] = cpu_instruction!(SET_3_r_c);
    cb_dispatch_array[0xDA] = cpu_instruction!(SET_3_r_d);
    cb_dispatch_array[0xDB] = cpu_instruction!(SET_3_r_e);
    cb_dispatch_array[0xDC] = cpu_instruction!(SET_3_r_h);
    cb_dispatch_array[0xDD] = cpu_instruction!(SET_3_r_l);
    cb_dispatch_array[0xDE] = cpu_instruction!(SET_3_HLm);
    cb_dispatch_array[0xDF] = cpu_instruction!(SET_3_r_a);

    cb_dispatch_array[0xE0] = cpu_instruction!(SET_4_r_b);
    cb_dispatch_array[0xE1] = cpu_instruction!(SET_4_r_c);
    cb_dispatch_array[0xE2] = cpu_instruction!(SET_4_r_d);
    cb_dispatch_array[0xE3] = cpu_instruction!(SET_4_r_e);
    cb_dispatch_array[0xE4] = cpu_instruction!(SET_4_r_h);
    cb_dispatch_array[0xE5] = cpu_instruction!(SET_4_r_l);
    cb_dispatch_array[0xE6] = cpu_instruction!(SET_4_HLm);
    cb_dispatch_array[0xE7] = cpu_instruction!(SET_4_r_a);
    cb_dispatch_array[0xE8] = cpu_instruction!(SET_5_r_b);
    cb_dispatch_array[0xE9] = cpu_instruction!(SET_5_r_c);
    cb_dispatch_array[0xEA] = cpu_instruction!(SET_5_r_d);
    cb_dispatch_array[0xEB] = cpu_instruction!(SET_5_r_e);
    cb_dispatch_array[0xEC] = cpu_instruction!(SET_5_r_h);
    cb_dispatch_array[0xED] = cpu_instruction!(SET_5_r_l);
    cb_dispatch_array[0xEE] = cpu_instruction!(SET_5_HLm);
    cb_dispatch_array[0xEF] = cpu_instruction!(SET_5_r_a);

    cb_dispatch_array[0xF0] = cpu_instruction!(SET_6_r_b);
    cb_dispatch_array[0xF1] = cpu_instruction!(SET_6_r_c);
    cb_dispatch_array[0xF2] = cpu_instruction!(SET_6_r_d);
    cb_dispatch_array[0xF3] = cpu_instruction!(SET_6_r_e);
    cb_dispatch_array[0xF4] = cpu_instruction!(SET_6_r_h);
    cb_dispatch_array[0xF5] = cpu_instruction!(SET_6_r_l);
    cb_dispatch_array[0xF6] = cpu_instruction!(SET_6_HLm);
    cb_dispatch_array[0xF7] = cpu_instruction!(SET_6_r_a);
    cb_dispatch_array[0xF8] = cpu_instruction!(SET_7_r_b);
    cb_dispatch_array[0xF9] = cpu_instruction!(SET_7_r_c);
    cb_dispatch_array[0xFA] = cpu_instruction!(SET_7_r_d);
    cb_dispatch_array[0xFB] = cpu_instruction!(SET_7_r_e);
    cb_dispatch_array[0xFC] = cpu_instruction!(SET_7_r_h);
    cb_dispatch_array[0xFD] = cpu_instruction!(SET_7_r_l);
    cb_dispatch_array[0xFE] = cpu_instruction!(SET_7_HLm);
    cb_dispatch_array[0xFF] = cpu_instruction!(SET_7_r_a);

    cb_dispatch_array
}
