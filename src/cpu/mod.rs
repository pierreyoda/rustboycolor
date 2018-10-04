/// Module emulating the behavior of the Sharp LR35902 processor powering the
/// Game Boy (Color).

mod ops;
mod cb_ops;
#[cfg(test)] mod test;

use irq::{Interrupt, INTERRUPT_FLAG_ADDRESS, INTERRUPT_ENABLE_ADDRESS};
use memory::Memory;
use registers::{Registers, Z_FLAG, N_FLAG, H_FLAG, C_FLAG};

/// The CPU clock speed for the Game Boy (Classic), in Hz.
pub const CPU_CLOCK_SPEED: u32 = 4_194_304;

/// The type used to count the CPU's machine cycles.
/// 1 machine cycle = 4 clock cycles
pub type CycleType = u64;

/// The structure holding and emulating the CPU state.
pub struct Cpu<M> {
    /// The number of CPU cycles spent since the start of the emulation.
    cycles: CycleType,
    /// Is the CPU execution halted ?
    pub halted: bool,
    /// The CPU's registers.
    pub regs: Registers,
    /// The memory on which the CPU operates.
    pub mem: M,
    /// Interrupt master enable switch.
    pub ime: bool,
    /// Interrupt flag register before CPU HALT.
    pub if_reg_before_halt: u8,
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
            halted: false,
            regs: Registers::new(),
            mem,
            ime: true,
            if_reg_before_halt: 0x00,
            opcode: 0x0,
            dispatch_array: dispatch_array(),
            cb_dispatch_array: cb_dispatch_array(),
        }
    }

    /// Simulate the effects of the power-up sequence.
    /// Source: http://gbdev.gg8.se/wiki/articles/Power_Up_Sequence
    pub fn post_bios(&mut self) {
        self.regs.set_af(0x01B0);
        self.regs.set_bc(0x0013);
        self.regs.set_de(0x00D8);
        self.regs.set_hl(0x014D);
        self.regs.pc = 0x100;
        self.regs.sp = 0xFFFE;
        self.mem.write_byte(0xFF10, 0x80);
        self.mem.write_byte(0xFF11, 0xBF);
        self.mem.write_byte(0xFF12, 0xF3);
        self.mem.write_byte(0xFF14, 0xBF);
        self.mem.write_byte(0xFF16, 0x3F);
        self.mem.write_byte(0xFF19, 0xBF);
        self.mem.write_byte(0xFF1A, 0x7F);
        self.mem.write_byte(0xFF1B, 0xFF);
        self.mem.write_byte(0xFF1C, 0x9F);
        self.mem.write_byte(0xFF1E, 0xBF);
        self.mem.write_byte(0xFF20, 0xFF);
        self.mem.write_byte(0xFF23, 0xBF);
        self.mem.write_byte(0xFF24, 0x77);
        self.mem.write_byte(0xFF25, 0xF3);
        self.mem.write_byte(0xFF26, 0xF1);
        self.mem.write_byte(0xFF40, 0x91);
        self.mem.write_byte(0xFF47, 0xFC);
        self.mem.write_byte(0xFF48, 0xFF);
        self.mem.write_byte(0xFF49, 0xFF);
    }

    /// Get an immutable reference to the Registers state.
    pub fn registers(&self) -> &Registers {
        &self.regs
    }

    /// Fetch the next byte in memory.
    fn fetch_byte(&mut self) -> u8 {
        let b = self.mem.read_byte(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        b
    }
    /// Fetch the next word in memory.
    fn fetch_word(&mut self) -> u16 {
        let w = self.mem.read_word(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(2);
        w
    }

    /// Advance the CPU simulation and return the number of CPU machine cycles
    /// spent.
    pub fn step(&mut self) -> CycleType {
        if self.halted {
            // if any interrupt occured, resume execution
            let if_reg = self.mem.read_byte(INTERRUPT_FLAG_ADDRESS);
            if self.if_reg_before_halt != if_reg { self.halted = false; }
            return 1; // NOP
        }
        let mut step_cycles = self.handle_interrupt();
        self.opcode = self.fetch_byte();
        // println!("OP={:0>2X} PC={:0>4X} SP={:0>4X}", self.opcode, self.regs.pc, self.regs.sp);
        step_cycles += self.dispatch_array[self.opcode as usize](self);
        self.cycles += step_cycles;
        step_cycles
    }

    fn handle_interrupt(&mut self) -> CycleType {
        if !self.ime { return 0; }
        let ie_reg = self.mem.read_byte(INTERRUPT_ENABLE_ADDRESS);
        let if_reg = self.mem.read_byte(INTERRUPT_FLAG_ADDRESS);
        let interrupts = ie_reg & if_reg;
        if interrupts == 0x00 { return 0; }
        // check the interrupts by order of priority
        for i in 0..5 {
            let interrupt_vector = match Interrupt::from_u8(
                interrupts & (1 << i)) {
                Some(interrupt) => interrupt.address(),
                None            => continue,
            };
            self.mem.write_byte(INTERRUPT_FLAG_ADDRESS, interrupts & !(1 << i));
            self.ime = false;
            self.cpu_call(interrupt_vector);
        }

        // TO CHECK : cycles count
        0
    }

    /// Called when encountering the '0xCB' prefix.
    pub fn call_cb(&mut self) -> CycleType {
        self.opcode = self.fetch_byte();
        self.cb_dispatch_array[self.opcode as usize](self)
    }

    /// Called when an unknown opcode is encountered.
    /// TODO improved behavior
    pub fn opcode_unknown(&mut self) -> CycleType {
        warn!("CPU : unknown opcode 0x{:0>2X} ; halting",
                 self.opcode);
        self.halted = true;
        0
    }
    /// Called when an unknown CB-prefixed opcode is encountered.
    /// TODO improved behavior
    pub fn cb_opcode_unknown(&mut self) -> CycleType {
        warn!("CPU : unknown opcode 0xCB{:0>2X} ; halting",
                 self.opcode);
        self.halted = true;
        0
    }

    /// Push a 16-bit value to the stack.
    /// NB : on this Z80-derived CPU, the stack grows from top down
    fn stack_push(&mut self, value: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.mem.write_word(self.regs.sp, value);
    }
    /// Pop a 16-bit value from the stack.
    fn stack_pop(&mut self) -> u16 {
        let value = self.mem.read_word(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        value
    }

    /// Call the subroutine at the given address.
    fn cpu_call(&mut self, address: u16) {
        let pc = self.regs.pc;
        self.stack_push(pc);
        self.regs.pc = address;
    }

    /// Perform a relative jump by the signed immediate byte.
    fn cpu_jr(&mut self, b: u8) -> CycleType {
        let n = b as i8 as i16 as u16;
        self.regs.pc = self.regs.pc.wrapping_add(n);
        3
    }

    /// Add a 16-bit value to another one.
    fn alu_add16(&mut self, a: u16, b: u16) -> u16 {
        let r = (a as u32) + (b as u32);
        self.regs.set_flag(N_FLAG, false);
        self.regs.set_flag(H_FLAG, (a & 0x0FFF) + (b & 0x0FFF) > 0x0FFF);
        self.regs.set_flag(C_FLAG, r > 0xFFFF);
        r as u16
    }

    /// Add 'b' (and C if add_c is true) to register A.
    fn alu_add(&mut self, b: u8, add_c: bool) {
        let a = self.regs.a;
        let c = if add_c && self.regs.flag(C_FLAG) { 1 } else { 0 };
        let r = (a as u16) + (b as u16) + (c as u16);
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(N_FLAG, false);
        self.regs.set_flag(H_FLAG, (a & 0x0F) + (b & 0x0F) + c > 0x0F);
        self.regs.set_flag(C_FLAG, r > 0xFF);
        self.regs.a = r as u8;
    }

    /// Substract 'b' (and C if sub_c is true) from register A.
    fn alu_sub(&mut self, b: u8, sub_c: bool) {
        let a = self.regs.a;
        let c = if sub_c && self.regs.flag(C_FLAG) { 1 } else { 0 };
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(N_FLAG, true);
        self.regs.set_flag(H_FLAG, (a & 0x0F) < (b & 0x0F) + c);
        self.regs.set_flag(C_FLAG, (a  as u16) < (b as u16) + (c as u16));
        self.regs.a = r;
    }

    /// Logical AND against register A.
    fn alu_and(&mut self, b: u8) {
        let r = self.regs.a & b;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(H_FLAG, true);
        self.regs.set_flag(N_FLAG | C_FLAG, false);
        self.regs.a = r;
    }
    /// Logical OR against register A.
    fn alu_or(&mut self, b: u8) {
        let r = self.regs.a | b;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(N_FLAG | H_FLAG | C_FLAG, false);
        self.regs.a = r;
    }
    /// Logical XOR against register A.
    fn alu_xor(&mut self, b: u8) {
        let r = self.regs.a ^ b;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(N_FLAG | H_FLAG | C_FLAG, false);
        self.regs.a = r;
    }
    /// Compare against register A.
    fn alu_cp(&mut self, b: u8) {
        let a = self.regs.a;
        // use alu_sub to set the flags properly
        self.alu_sub(b, false);
        // and restore the value of register A
        self.regs.a = a;
    }

    /// Rotate left.
    fn alu_rl(&mut self, v: u8) -> u8 {
        let c  = (v & 0x80) == 0x80;
        let r = (v << 1) | (if self.regs.flag(C_FLAG) { 0x01 } else { 0x00 });
        self.regs.f = 0;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(C_FLAG, c);
        r
    }
    /// Rotate left with carry.
    fn alu_rlc(&mut self, v: u8) -> u8 {
        let c  = (v & 0x80) == 0x80;
        let r = (v << 1) | (if c { 0x01 } else { 0x00 });
        self.regs.f = 0;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(C_FLAG, c);
        r
    }

    /// Rotate right.
    fn alu_rr(&mut self, v: u8) -> u8 {
        let c  = (v & 0x01) == 0x01;
        let r = (v >> 1) | (if self.regs.flag(C_FLAG) { 0x80 } else { 0x00 });
        self.regs.f = 0;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(C_FLAG, c);
        r
    }
    /// Rotate right with carry.
    fn alu_rrc(&mut self, v: u8) -> u8 {
        let c  = (v & 0x01) == 0x01;
        let r = (v >> 1) | (if c { 0x80 } else { 0x00 });
        self.regs.f = 0;
        self.regs.set_flag(Z_FLAG, r == 0x0);
        self.regs.set_flag(C_FLAG, c);
        r
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
/// http://www.z80.info/index.htm
/// http://z80-heaven.wikidot.com/
fn dispatch_array<M: Memory>() -> [CpuInstruction<M>; 256] {
    // the default value is the method opcode_unknown
    let mut dispatch_array = [Cpu::<M>::opcode_unknown as CpuInstruction<M>; 256];

    dispatch_array[0x00] = cpu_instruction!(NOP);
    dispatch_array[0x01] = cpu_instruction!(LD_BC_nn);
    dispatch_array[0x02] = cpu_instruction!(LD_BCm_A);
    dispatch_array[0x03] = cpu_instruction!(INC_BC);
    dispatch_array[0x04] = cpu_instruction!(INC_r_b);
    dispatch_array[0x05] = cpu_instruction!(DEC_r_b);
    dispatch_array[0x06] = cpu_instruction!(LD_r_n_b);
    dispatch_array[0x07] = cpu_instruction!(RLC);
    dispatch_array[0x08] = cpu_instruction!(LD_NNm_SP);
    dispatch_array[0x09] = cpu_instruction!(ADD_HL_BC);
    dispatch_array[0x0A] = cpu_instruction!(LD_A_BCm);
    dispatch_array[0x0B] = cpu_instruction!(DEC_BC);
    dispatch_array[0x0C] = cpu_instruction!(INC_r_c);
    dispatch_array[0x0D] = cpu_instruction!(DEC_r_c);
    dispatch_array[0x0E] = cpu_instruction!(LD_r_n_c);
    dispatch_array[0x0F] = cpu_instruction!(RRC);

    dispatch_array[0x10] = cpu_instruction!(STOP);
    dispatch_array[0x11] = cpu_instruction!(LD_DE_nn);
    dispatch_array[0x12] = cpu_instruction!(LD_DEm_A);
    dispatch_array[0x13] = cpu_instruction!(INC_DE);
    dispatch_array[0x14] = cpu_instruction!(INC_r_d);
    dispatch_array[0x15] = cpu_instruction!(DEC_r_d);
    dispatch_array[0x16] = cpu_instruction!(LD_r_n_d);
    dispatch_array[0x17] = cpu_instruction!(RL);
    dispatch_array[0x18] = cpu_instruction!(JR_n);
    dispatch_array[0x19] = cpu_instruction!(ADD_HL_DE);
    dispatch_array[0x1A] = cpu_instruction!(LD_A_DEm);
    dispatch_array[0x1B] = cpu_instruction!(DEC_DE);
    dispatch_array[0x1C] = cpu_instruction!(INC_r_e);
    dispatch_array[0x1D] = cpu_instruction!(DEC_r_e);
    dispatch_array[0x1E] = cpu_instruction!(LD_r_n_e);
    dispatch_array[0x1F] = cpu_instruction!(RR);

    dispatch_array[0x20] = cpu_instruction!(JR_NZ_n);
    dispatch_array[0x21] = cpu_instruction!(LD_HL_nn);
    dispatch_array[0x22] = cpu_instruction!(LDI_HLm_A);
    dispatch_array[0x23] = cpu_instruction!(INC_HL);
    dispatch_array[0x24] = cpu_instruction!(INC_r_h);
    dispatch_array[0x25] = cpu_instruction!(DEC_r_h);
    dispatch_array[0x26] = cpu_instruction!(LD_r_n_h);
    dispatch_array[0x27] = cpu_instruction!(DAA);
    dispatch_array[0x28] = cpu_instruction!(JR_Z_n);
    dispatch_array[0x29] = cpu_instruction!(ADD_HL_HL);
    dispatch_array[0x2A] = cpu_instruction!(LDI_A_HLm);
    dispatch_array[0x2B] = cpu_instruction!(DEC_HL);
    dispatch_array[0x2C] = cpu_instruction!(INC_r_l);
    dispatch_array[0x2D] = cpu_instruction!(DEC_r_l);
    dispatch_array[0x2E] = cpu_instruction!(LD_r_n_l);
    dispatch_array[0x2F] = cpu_instruction!(CPL);

    dispatch_array[0x30] = cpu_instruction!(JR_NC_n);
    dispatch_array[0x31] = cpu_instruction!(LD_SP_nn);
    dispatch_array[0x32] = cpu_instruction!(LDD_HLm_A);
    dispatch_array[0x33] = cpu_instruction!(INC_SP);
    dispatch_array[0x34] = cpu_instruction!(INC_HLm);
    dispatch_array[0x35] = cpu_instruction!(DEC_HLm);
    dispatch_array[0x36] = cpu_instruction!(LD_HLm_n);
    dispatch_array[0x37] = cpu_instruction!(SCF);
    dispatch_array[0x38] = cpu_instruction!(JR_C_n);
    dispatch_array[0x39] = cpu_instruction!(ADD_HL_SP);
    dispatch_array[0x3A] = cpu_instruction!(LDD_A_HLm);
    dispatch_array[0x3B] = cpu_instruction!(DEC_SP);
    dispatch_array[0x3C] = cpu_instruction!(INC_r_a);
    dispatch_array[0x3D] = cpu_instruction!(DEC_r_a);
    dispatch_array[0x3E] = cpu_instruction!(LD_r_n_a);
    dispatch_array[0x3F] = cpu_instruction!(CCF);

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
    dispatch_array[0x76] = cpu_instruction!(HALT);
    dispatch_array[0x77] = cpu_instruction!(LD_HLm_r_a);
    dispatch_array[0x78] = cpu_instruction!(LD_rr_ab);
    dispatch_array[0x79] = cpu_instruction!(LD_rr_ac);
    dispatch_array[0x7A] = cpu_instruction!(LD_rr_ad);
    dispatch_array[0x7B] = cpu_instruction!(LD_rr_ae);
    dispatch_array[0x7C] = cpu_instruction!(LD_rr_ah);
    dispatch_array[0x7D] = cpu_instruction!(LD_rr_al);
    dispatch_array[0x7E] = cpu_instruction!(LD_r_HLm_a);
    dispatch_array[0x7F] = cpu_instruction!(LD_rr_aa);

    dispatch_array[0x80] = cpu_instruction!(ADD_r_b);
    dispatch_array[0x81] = cpu_instruction!(ADD_r_c);
    dispatch_array[0x82] = cpu_instruction!(ADD_r_d);
    dispatch_array[0x83] = cpu_instruction!(ADD_r_e);
    dispatch_array[0x84] = cpu_instruction!(ADD_r_h);
    dispatch_array[0x85] = cpu_instruction!(ADD_r_l);
    dispatch_array[0x86] = cpu_instruction!(ADD_HLm);
    dispatch_array[0x87] = cpu_instruction!(ADD_r_a);
    dispatch_array[0x88] = cpu_instruction!(ADC_r_b);
    dispatch_array[0x89] = cpu_instruction!(ADC_r_c);
    dispatch_array[0x8A] = cpu_instruction!(ADC_r_d);
    dispatch_array[0x8B] = cpu_instruction!(ADC_r_e);
    dispatch_array[0x8C] = cpu_instruction!(ADC_r_h);
    dispatch_array[0x8D] = cpu_instruction!(ADC_r_l);
    dispatch_array[0x8E] = cpu_instruction!(ADC_HLm);
    dispatch_array[0x8F] = cpu_instruction!(ADC_r_a);

    dispatch_array[0x90] = cpu_instruction!(SUB_r_b);
    dispatch_array[0x91] = cpu_instruction!(SUB_r_c);
    dispatch_array[0x92] = cpu_instruction!(SUB_r_d);
    dispatch_array[0x93] = cpu_instruction!(SUB_r_e);
    dispatch_array[0x94] = cpu_instruction!(SUB_r_h);
    dispatch_array[0x95] = cpu_instruction!(SUB_r_l);
    dispatch_array[0x96] = cpu_instruction!(SUB_HLm);
    dispatch_array[0x97] = cpu_instruction!(SUB_r_a);
    dispatch_array[0x98] = cpu_instruction!(SBC_r_b);
    dispatch_array[0x99] = cpu_instruction!(SBC_r_c);
    dispatch_array[0x9A] = cpu_instruction!(SBC_r_d);
    dispatch_array[0x9B] = cpu_instruction!(SBC_r_e);
    dispatch_array[0x9C] = cpu_instruction!(SBC_r_h);
    dispatch_array[0x9D] = cpu_instruction!(SBC_r_l);
    dispatch_array[0x9E] = cpu_instruction!(SBC_HLm);
    dispatch_array[0x9F] = cpu_instruction!(SBC_r_a);

    dispatch_array[0xA0] = cpu_instruction!(AND_r_b);
    dispatch_array[0xA1] = cpu_instruction!(AND_r_c);
    dispatch_array[0xA2] = cpu_instruction!(AND_r_d);
    dispatch_array[0xA3] = cpu_instruction!(AND_r_e);
    dispatch_array[0xA4] = cpu_instruction!(AND_r_h);
    dispatch_array[0xA5] = cpu_instruction!(AND_r_l);
    dispatch_array[0xA6] = cpu_instruction!(AND_HLm);
    dispatch_array[0xA7] = cpu_instruction!(AND_r_a);
    dispatch_array[0xA8] = cpu_instruction!(XOR_r_b);
    dispatch_array[0xA9] = cpu_instruction!(XOR_r_c);
    dispatch_array[0xAA] = cpu_instruction!(XOR_r_d);
    dispatch_array[0xAB] = cpu_instruction!(XOR_r_e);
    dispatch_array[0xAC] = cpu_instruction!(XOR_r_h);
    dispatch_array[0xAD] = cpu_instruction!(XOR_r_l);
    dispatch_array[0xAE] = cpu_instruction!(XOR_HLm);
    dispatch_array[0xAF] = cpu_instruction!(XOR_r_a);

    dispatch_array[0xB0] = cpu_instruction!(OR_r_b);
    dispatch_array[0xB1] = cpu_instruction!(OR_r_c);
    dispatch_array[0xB2] = cpu_instruction!(OR_r_d);
    dispatch_array[0xB3] = cpu_instruction!(OR_r_e);
    dispatch_array[0xB4] = cpu_instruction!(OR_r_h);
    dispatch_array[0xB5] = cpu_instruction!(OR_r_l);
    dispatch_array[0xB6] = cpu_instruction!(OR_HLm);
    dispatch_array[0xB7] = cpu_instruction!(OR_r_a);
    dispatch_array[0xB8] = cpu_instruction!(CP_r_b);
    dispatch_array[0xB9] = cpu_instruction!(CP_r_c);
    dispatch_array[0xBA] = cpu_instruction!(CP_r_d);
    dispatch_array[0xBB] = cpu_instruction!(CP_r_e);
    dispatch_array[0xBC] = cpu_instruction!(CP_r_h);
    dispatch_array[0xBD] = cpu_instruction!(CP_r_l);
    dispatch_array[0xBE] = cpu_instruction!(CP_HLm);
    dispatch_array[0xBF] = cpu_instruction!(CP_r_a);

    dispatch_array[0xC0] = cpu_instruction!(RET_NZ);
    dispatch_array[0xC1] = cpu_instruction!(POP_BC);
    dispatch_array[0xC2] = cpu_instruction!(JP_NZ_nn);
    dispatch_array[0xC3] = cpu_instruction!(JP_nn);
    dispatch_array[0xC4] = cpu_instruction!(CALL_NZ_nn);
    dispatch_array[0xC5] = cpu_instruction!(PUSH_BC);
    dispatch_array[0xC6] = cpu_instruction!(ADD_n);
    dispatch_array[0xC7] = cpu_instruction!(RST_00H);
    dispatch_array[0xC8] = cpu_instruction!(RET_Z);
    dispatch_array[0xC9] = cpu_instruction!(RET);
    dispatch_array[0xCA] = cpu_instruction!(JP_Z_nn);
    dispatch_array[0xCB] = cpu_instruction!(call_cb);
    dispatch_array[0xCC] = cpu_instruction!(CALL_Z_nn);
    dispatch_array[0xCD] = cpu_instruction!(CALL_nn);
    dispatch_array[0xCE] = cpu_instruction!(ADC_n);
    dispatch_array[0xCF] = cpu_instruction!(RST_08H);

    dispatch_array[0xD0] = cpu_instruction!(RET_NC);
    dispatch_array[0xD1] = cpu_instruction!(POP_DE);
    dispatch_array[0xD2] = cpu_instruction!(JP_NC_nn);
    dispatch_array[0xD3] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xD4] = cpu_instruction!(CALL_NC_nn);
    dispatch_array[0xD5] = cpu_instruction!(PUSH_DE);
    dispatch_array[0xD6] = cpu_instruction!(SUB_n);
    dispatch_array[0xD7] = cpu_instruction!(RST_10H);
    dispatch_array[0xD8] = cpu_instruction!(RET_C);
    dispatch_array[0xD9] = cpu_instruction!(RETI);
    dispatch_array[0xDA] = cpu_instruction!(JP_C_nn);
    dispatch_array[0xDB] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xDC] = cpu_instruction!(CALL_C_nn);
    dispatch_array[0xDD] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xDE] = cpu_instruction!(SBC_n);
    dispatch_array[0xDF] = cpu_instruction!(RST_18H);

    dispatch_array[0xE0] = cpu_instruction!(LDH_n_A);
    dispatch_array[0xE1] = cpu_instruction!(POP_HL);
    dispatch_array[0xE2] = cpu_instruction!(LDH_C_A);
    dispatch_array[0xE3] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xE4] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xE5] = cpu_instruction!(PUSH_HL);
    dispatch_array[0xE6] = cpu_instruction!(AND_n);
    dispatch_array[0xE7] = cpu_instruction!(RST_20H);
    dispatch_array[0xE8] = cpu_instruction!(ADD_SP_n);
    dispatch_array[0xE9] = cpu_instruction!(JP_HLm);
    dispatch_array[0xEA] = cpu_instruction!(LD_NNm_A);
    dispatch_array[0xEB] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xEC] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xED] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xEE] = cpu_instruction!(XOR_n);
    dispatch_array[0xEF] = cpu_instruction!(RST_28H);

    dispatch_array[0xF0] = cpu_instruction!(LDH_A_n);
    dispatch_array[0xF1] = cpu_instruction!(POP_AF);
    dispatch_array[0xF2] = cpu_instruction!(LDH_A_C);
    dispatch_array[0xF3] = cpu_instruction!(DI);
    dispatch_array[0xF4] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xF5] = cpu_instruction!(PUSH_AF);
    dispatch_array[0xF6] = cpu_instruction!(OR_n);
    dispatch_array[0xF7] = cpu_instruction!(RST_30H);
    dispatch_array[0xF8] = cpu_instruction!(LDHL_SP_n);
    dispatch_array[0xF9] = cpu_instruction!(LD_SP_HL);
    dispatch_array[0xFA] = cpu_instruction!(LD_A_NNm);
    dispatch_array[0xFB] = cpu_instruction!(EI);
    dispatch_array[0xFC] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xFD] = cpu_instruction!(opcode_unknown);
    dispatch_array[0xFE] = cpu_instruction!(CP_n);
    dispatch_array[0xFF] = cpu_instruction!(RST_38H);

    dispatch_array
}

/// Same as 'dispatch_array()' but for the additional, CB-prefixed instructions.
fn cb_dispatch_array<M: Memory>() -> [CpuInstruction<M>; 256] {
    // the default value is the method cb_opcode_unknown
    let mut cb_dispatch_array = [Cpu::<M>::cb_opcode_unknown
        as CpuInstruction<M>; 256];

    cb_dispatch_array[0x00] = cpu_instruction!(RLC_r_b);
    cb_dispatch_array[0x01] = cpu_instruction!(RLC_r_c);
    cb_dispatch_array[0x02] = cpu_instruction!(RLC_r_d);
    cb_dispatch_array[0x03] = cpu_instruction!(RLC_r_e);
    cb_dispatch_array[0x04] = cpu_instruction!(RLC_r_h);
    cb_dispatch_array[0x05] = cpu_instruction!(RLC_r_l);
    cb_dispatch_array[0x06] = cpu_instruction!(RLC_HLm);
    cb_dispatch_array[0x07] = cpu_instruction!(RLC_r_a);
    cb_dispatch_array[0x08] = cpu_instruction!(RRC_r_b);
    cb_dispatch_array[0x09] = cpu_instruction!(RRC_r_c);
    cb_dispatch_array[0x0A] = cpu_instruction!(RRC_r_d);
    cb_dispatch_array[0x0B] = cpu_instruction!(RRC_r_e);
    cb_dispatch_array[0x0C] = cpu_instruction!(RRC_r_h);
    cb_dispatch_array[0x0D] = cpu_instruction!(RRC_r_l);
    cb_dispatch_array[0x0E] = cpu_instruction!(RRC_HLm);
    cb_dispatch_array[0x0F] = cpu_instruction!(RRC_r_a);

    cb_dispatch_array[0x10] = cpu_instruction!(RL_r_b);
    cb_dispatch_array[0x11] = cpu_instruction!(RL_r_c);
    cb_dispatch_array[0x12] = cpu_instruction!(RL_r_d);
    cb_dispatch_array[0x13] = cpu_instruction!(RL_r_e);
    cb_dispatch_array[0x14] = cpu_instruction!(RL_r_h);
    cb_dispatch_array[0x15] = cpu_instruction!(RL_r_l);
    cb_dispatch_array[0x16] = cpu_instruction!(RL_HLm);
    cb_dispatch_array[0x17] = cpu_instruction!(RL_r_a);
    cb_dispatch_array[0x18] = cpu_instruction!(RR_r_b);
    cb_dispatch_array[0x19] = cpu_instruction!(RR_r_c);
    cb_dispatch_array[0x1A] = cpu_instruction!(RR_r_d);
    cb_dispatch_array[0x1B] = cpu_instruction!(RR_r_e);
    cb_dispatch_array[0x1C] = cpu_instruction!(RR_r_h);
    cb_dispatch_array[0x1D] = cpu_instruction!(RR_r_l);
    cb_dispatch_array[0x1E] = cpu_instruction!(RR_HLm);
    cb_dispatch_array[0x1F] = cpu_instruction!(RR_r_a);

    cb_dispatch_array[0x20] = cpu_instruction!(SLA_r_b);
    cb_dispatch_array[0x21] = cpu_instruction!(SLA_r_c);
    cb_dispatch_array[0x22] = cpu_instruction!(SLA_r_d);
    cb_dispatch_array[0x23] = cpu_instruction!(SLA_r_e);
    cb_dispatch_array[0x24] = cpu_instruction!(SLA_r_h);
    cb_dispatch_array[0x25] = cpu_instruction!(SLA_r_l);
    cb_dispatch_array[0x26] = cpu_instruction!(SLA_HLm);
    cb_dispatch_array[0x27] = cpu_instruction!(SLA_r_a);
    cb_dispatch_array[0x28] = cpu_instruction!(SRA_r_b);
    cb_dispatch_array[0x29] = cpu_instruction!(SRA_r_c);
    cb_dispatch_array[0x2A] = cpu_instruction!(SRA_r_d);
    cb_dispatch_array[0x2B] = cpu_instruction!(SRA_r_e);
    cb_dispatch_array[0x2C] = cpu_instruction!(SRA_r_h);
    cb_dispatch_array[0x2D] = cpu_instruction!(SRA_r_l);
    cb_dispatch_array[0x2E] = cpu_instruction!(SRA_HLm);
    cb_dispatch_array[0x2F] = cpu_instruction!(SRA_r_a);

    cb_dispatch_array[0x30] = cpu_instruction!(SWAP_r_b);
    cb_dispatch_array[0x31] = cpu_instruction!(SWAP_r_c);
    cb_dispatch_array[0x32] = cpu_instruction!(SWAP_r_d);
    cb_dispatch_array[0x33] = cpu_instruction!(SWAP_r_e);
    cb_dispatch_array[0x34] = cpu_instruction!(SWAP_r_h);
    cb_dispatch_array[0x35] = cpu_instruction!(SWAP_r_l);
    cb_dispatch_array[0x36] = cpu_instruction!(SWAP_HLm);
    cb_dispatch_array[0x37] = cpu_instruction!(SWAP_r_a);
    cb_dispatch_array[0x38] = cpu_instruction!(SRL_r_b);
    cb_dispatch_array[0x39] = cpu_instruction!(SRL_r_c);
    cb_dispatch_array[0x3A] = cpu_instruction!(SRL_r_d);
    cb_dispatch_array[0x3B] = cpu_instruction!(SRL_r_e);
    cb_dispatch_array[0x3C] = cpu_instruction!(SRL_r_h);
    cb_dispatch_array[0x3D] = cpu_instruction!(SRL_r_l);
    cb_dispatch_array[0x3E] = cpu_instruction!(SRL_HLm);
    cb_dispatch_array[0x3F] = cpu_instruction!(SRL_r_a);

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
