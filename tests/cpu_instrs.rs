use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use rustboylib::cpu::Cpu;
use rustboylib::mbc;
use rustboylib::mmu::MMU;
use rustboylib::serial::SerialCallback;

const CPU_STEPS_LIMIT: usize = 10_000_000; // TODO: detect test ending?

fn setup(rom_path: &str) -> (Cpu<MMU>, Rc<RefCell<String>>) {
    let serial_output = Rc::new(RefCell::new(String::new()));
    let serial_output_mmu = serial_output.clone();
    let serial_callback: SerialCallback =
        Box::new(move |data: u8| serial_output_mmu.borrow_mut().push(data as char));

    let mbc = mbc::load_cartridge(Path::new(rom_path)).expect("test ROM loading error");
    let mmu = MMU::new(mbc, false, true, Some(serial_callback));
    let mut cpu = Cpu::new(mmu);
    cpu.post_bios();
    (cpu, serial_output)
}

macro_rules! test_blargg_cpu_instrs {
    ($ ( $name: ident : $filename: expr, )* ) => {
    $(
        #[test]
        fn $name() {
            let rom_path = format!("tests/cpu_instrs/{}.gb", $filename);
            let (mut cpu, serial_output) = setup(&rom_path);
            for _ in 0..CPU_STEPS_LIMIT {
                cpu.step();
            }
            println!("blargg cpu test '{}' serial output:\n{}", $filename, serial_output.borrow());
            assert!(serial_output.borrow().contains("Passed"), "'{}' does not pass", $filename);
        }
    )*
    }
}

test_blargg_cpu_instrs! {
    test_cpu_instrs_01: "01-special",
    test_cpu_instrs_02: "02-interrupts",
    test_cpu_instrs_03: "03-op sp,hl",
    test_cpu_instrs_04: "04-op r,imm",
    test_cpu_instrs_05: "05-op rp",
    test_cpu_instrs_06: "06-ld r,r",
    test_cpu_instrs_07: "07-jr,jp,call,ret,rst",
    test_cpu_instrs_08: "08-misc instrs",
    test_cpu_instrs_09: "09-op r,r",
    test_cpu_instrs_10: "10-bit ops",
    test_cpu_instrs_11: "11-op a,(hl)",
}
