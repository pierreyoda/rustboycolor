extern crate rustboylib;

/// Simple CPU instruction testing facility leveraging the 'Memory' trait to
/// sequentially feed an array of opcodes to a 'Cpu' instance.
struct MemCpuTest {
    ic           : usize,
    instructions : Vec<u8>,
}
impl rustboylib::memory::Memory for MemCpuTest {
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
    fn read_word(&mut self, address: u16) -> u16 { 0 }
    fn write_word(&mut self, address: u16, word: u16) { }
}

fn main() {
    // CPU crude test
    let test_opcodes = vec![
        0x51, // ldrr_dc
        0x62, // ldrr_hd
        0xCB, 0x31, // SWAP rC
        0xCB, 0xE7, // SET 4,rA
        0xCB, 0xC0, // SET 0,rB
        0xCB, 0xA7, // RES 4,rA
        0xCB, 0x80, // RES 0,rB
        0xCB, 0xA1, // RES 4,rC
    ];
    let test_memory = MemCpuTest {ic: 0, instructions: test_opcodes};
    let mut cpu = rustboylib::cpu::Cpu::new(test_memory);
    cpu.regs.c = 0xAB;
    println!("cpu regs : {:?}", cpu.registers());
    cpu.step();
    cpu.step();
    println!("cpu regs : {:?}", cpu.registers());

    println!("SWAP");
    cpu.step();
    println!("cpu regs : {:?}", cpu.registers());

    println!("SET");
    cpu.step();
    cpu.step();
    println!("cpu regs : {:?}", cpu.registers());

    println!("RES");
    cpu.step();
    cpu.step();
    cpu.step();
    println!("cpu regs : {:?}", cpu.registers());
}
