extern crate rustboylib;

fn main() {
    // CPU TEST
    let mapper = rustboylib::mmu::MMU;
    let cpu = rustboylib::cpu::Cpu::new(mapper);
    println!("cpu regs : {:?}", cpu.registers());
}
