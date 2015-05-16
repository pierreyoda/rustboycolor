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
            dispatch_array: dispatch_array()
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

    /// Called when an unknown opcode is encountered.
    /// TODO improved behavior
    pub fn opcode_unknown(&mut self) -> CycleType {
        println!("CPU : unknown opcode {:0>2x} ; stopping",
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
/// Main source for the opcodes :
/// http://imrannazar.com/Gameboy-Z80-Opcode-Map
fn dispatch_array<M: Memory>() -> [CpuInstruction<M>; 256] {
    // the default value is the method opcode_unknown
    let mut dispatch_array = [Cpu::<M>::opcode_unknown as CpuInstruction<M>; 256];

    dispatch_array[0x00] = cpu_instruction!(nop);

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

    dispatch_array
}

