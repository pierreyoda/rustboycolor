/// Crate emulating the behavior of the Sharp LR35902 processor powering the
/// Game Boy (Color).

use super::memory::Memory;
use super::registers::Registers;

/// The structure holding the CPU state.
struct Cpu<M> {
    /// The memory on which the CPU operates.
    mem: M,
    /// The CPU's registers.
    registers: Registers,
}

impl<M> Cpu<M> where M: Memory {

}
