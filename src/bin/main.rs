use std::error::Error;
#[macro_use]
extern crate log;
extern crate rustboylib;
use rustboylib::{cpu, memory};

mod logger;

/// Simple CPU instruction testing facility leveraging the 'Memory' trait to
/// sequentially feed an array of opcodes to a 'Cpu' instance.
struct MemCpuTest {
    ic           : usize,
    instructions : Vec<u8>,
}
impl memory::Memory for MemCpuTest {
    fn read_byte(&mut self, address: u16) -> u8 {
        if self.ic < self.instructions.len() {
            let opcode = self.instructions[self.ic];
            self.ic += 1;
            opcode
        } else {
            println!("MemCpuTest : instructions end reached");
            0x10
        }
    }
    fn write_byte(&mut self, address: u16, byte: u8) {}
}

fn print_cpu_registers<M: memory::Memory>(cpu: &cpu::Cpu<M>) {
    println!("cpu regs : {:?}", cpu.registers());
}

fn main() {
    // Logger initialization
    match logger::init_console_logger() {
        Err(error) => panic!(format!("Logging setup error : {}",
                                     error.description())),
        _ => (),
    }

    // CPU crude test
    let test_opcodes = vec![
        0x0E, 0xAB, // LD C, 0xAB
        0x51,       // LD D, C
        0x62,       // LD H, D
        0xCB, 0x31, // SWAP rC
        0xCB, 0xE7, // SET 4,rA
        0xCB, 0xC0, // SET 0,rB
        0xCB, 0xA7, // RES 4,rA
        0xCB, 0x80, // RES 0,rB
        0xCB, 0xA1, // RES 4,rC
        0xCB, 0x79, // BIT 7,rC
        0xCB, 0x72, // BIT 6,rD
        0xCB, 0x02, // RLC rD
    ];
    let test_memory = MemCpuTest {ic: 0, instructions: test_opcodes};
    let mut cpu = cpu::Cpu::new(test_memory);
    print_cpu_registers(&cpu);
    cpu.step();
    cpu.step();
    cpu.step();
    print_cpu_registers(&cpu);

    println!("SWAP C");
    cpu.step();
    print_cpu_registers(&cpu);

    println!("SET");
    cpu.step();
    cpu.step();
    print_cpu_registers(&cpu);

    println!("RES");
    cpu.step();
    cpu.step();
    cpu.step();
    print_cpu_registers(&cpu);

    println!("BIT 7,C");
    cpu.step();
    print_cpu_registers(&cpu);
    println!("BIT 6,D");
    cpu.step();
    print_cpu_registers(&cpu);

    println!("RLC D");
    cpu.step();
    print_cpu_registers(&cpu);
}
