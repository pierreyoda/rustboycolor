mod load; // load/store/move instructions

use super::super::memory::Memory;
use super::{Cpu, CycleType};

const OPCODE_END: u8 = 0xD3;

pub struct TestMachine {
    cpu: Cpu<TestMemory>,
}

impl TestMachine {
    pub fn with_instructions(instructions: &[u8]) -> Self {
        let mut m = TestMemory::new(0x10000);
        m.memory[0..instructions.len()].copy_from_slice(instructions);
        TestMachine { cpu: Cpu::new(m) }
    }

    pub fn init_cpu<F: Fn(&mut Cpu<TestMemory>) -> ()>(mut self, function: F) -> Self {
        function(&mut self.cpu);
        self
    }

    pub fn clock_cycles(&self) -> CycleType {
        self.cpu.cycles * 4
    }
}

pub fn test_cpu<F: Fn(&mut Cpu<TestMemory>) -> ()>(instructions: &[u8], init: F) -> TestMachine {
    let mut instrs = instructions.to_vec();
    instrs.push(OPCODE_END);
    let mut machine = TestMachine::with_instructions(&instrs).init_cpu(init);

    while machine.cpu.mem.memory[machine.cpu.regs.pc as usize] != OPCODE_END {
        machine.cpu.step();
    }

    machine
}

pub struct TestMemory {
    memory: Vec<u8>,
}

impl TestMemory {
    pub fn new(size: usize) -> Self {
        TestMemory {
            memory: vec![0x00; size],
        }
    }
}

impl Memory for TestMemory {
    fn read_byte(&mut self, address: u16) -> u8 {
        return self.memory[address as usize];
    }
    fn write_byte(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }
}
