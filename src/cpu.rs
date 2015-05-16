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
    /// The CPU's registers.
    pub regs: Registers,
    /// The memory on which the CPU operates.
    pub mem: M,
    /// The opcode currently executed.
    opcode: u8,
    /// The dispatching array used for instruction decoding.
    dispatch_array: [CpuInstruction<M>; 256]
}

impl<M> Cpu<M> where M: Memory {
    /// Return a new, initialized Cpu instance operating on the given 'Memory'.
    pub fn new(mem: M) -> Cpu<M> {
        Cpu {
            cycles: 0,
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
        println!("CPU : unknown opcode {:0>2x}", self.opcode);
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

    // --- LD ---
    dispatch_array[0x40] = cpu_instruction!(ldrr_bb);
    dispatch_array[0x41] = cpu_instruction!(ldrr_bc);
    dispatch_array[0x42] = cpu_instruction!(ldrr_bd);
    dispatch_array[0x43] = cpu_instruction!(ldrr_be);
    dispatch_array[0x44] = cpu_instruction!(ldrr_bh);
    dispatch_array[0x45] = cpu_instruction!(ldrr_bl);
    dispatch_array[0x50] = cpu_instruction!(ldrr_db);
    dispatch_array[0x51] = cpu_instruction!(ldrr_dc);
    dispatch_array[0x52] = cpu_instruction!(ldrr_dd);
    dispatch_array[0x53] = cpu_instruction!(ldrr_de);
    dispatch_array[0x54] = cpu_instruction!(ldrr_dh);
    dispatch_array[0x55] = cpu_instruction!(ldrr_dl);
    dispatch_array[0x60] = cpu_instruction!(ldrr_hb);
    dispatch_array[0x61] = cpu_instruction!(ldrr_hc);
    dispatch_array[0x62] = cpu_instruction!(ldrr_hd);
    dispatch_array[0x63] = cpu_instruction!(ldrr_he);
    dispatch_array[0x64] = cpu_instruction!(ldrr_hh);
    dispatch_array[0x65] = cpu_instruction!(ldrr_hl);
    // TODO LD H,(HL)
    dispatch_array[0x67] = cpu_instruction!(ldrr_ha);
    dispatch_array[0x68] = cpu_instruction!(ldrr_lb);
    dispatch_array[0x69] = cpu_instruction!(ldrr_lc);
    dispatch_array[0x6A] = cpu_instruction!(ldrr_ld);
    dispatch_array[0x6B] = cpu_instruction!(ldrr_le);
    dispatch_array[0x6C] = cpu_instruction!(ldrr_lh);
    dispatch_array[0x6D] = cpu_instruction!(ldrr_ll);
    // TODO LD L,(HL)
    dispatch_array[0x6F] = cpu_instruction!(ldrr_la);
    // TODO...
    dispatch_array[0x78] = cpu_instruction!(ldrr_ab);
    dispatch_array[0x79] = cpu_instruction!(ldrr_ac);
    dispatch_array[0x7A] = cpu_instruction!(ldrr_ad);
    dispatch_array[0x7B] = cpu_instruction!(ldrr_ae);
    dispatch_array[0x7C] = cpu_instruction!(ldrr_ah);
    dispatch_array[0x7D] = cpu_instruction!(ldrr_al);
    // TODO LD A,(HL)
    dispatch_array[0x7F] = cpu_instruction!(ldrr_aa);

    dispatch_array
}

