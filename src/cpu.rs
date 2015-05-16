/// Crate emulating the behavior of the Sharp LR35902 processor powering the
/// Game Boy (Color).

use super::memory::Memory;
use super::registers::Registers;

/// The structure holding and emulating the CPU state.
pub struct Cpu<M> {
    /// The number of CPU cycles spent since the start of the emulation.
    cycles: u64,
    /// The CPU's registers.
    regs: Registers,
    /// The memory on which the CPU operates.
    mem: M,
}

impl<M> Cpu<M> where M: Memory {
    /// Return a new, initialized Cpu instance operating on the given 'Memory'.
    pub fn new(mem: M) -> Cpu<M> {
        Cpu {
            cycles: 0,
            regs: Registers::new(),
            mem: mem
        }
    }

    /// Get an immutable reference to the Registers state.
    pub fn registers(&self) -> &Registers {
        &self.regs
    }
}
