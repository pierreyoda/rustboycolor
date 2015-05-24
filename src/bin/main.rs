extern crate rustboylib;

struct MemTest { i: u32 }
impl rustboylib::memory::Memory for MemTest {
    fn read_byte(&mut self, address: u16) -> u8 {
        self.i += 1;
        match self.i {
            1 => 0x51, // ldrr_dc
            2 => 0x62, // ldrr_hd
            3 => 0xCB,
            4 => 0x31, // SWAP rC
            5 => 0xCB,
            6 => 0xE7, // SET 4,rA
            7 => 0xCB,
            8 => 0xC0, // SET 0,rB
            _ => 0x10, // stop
        }
    }
    fn write_byte(&mut self, address: u16, byte: u8) {
    }
    fn read_word(&mut self, address: u16) -> u16 { 0 }
    fn write_word(&mut self, address: u16, word: u16) { }
}

fn main() {
    // CPU crude test
    let mapper = MemTest { i: 0}; //rustboylib::mmu::MMU;
    let mut cpu = rustboylib::cpu::Cpu::new(mapper);
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
}
