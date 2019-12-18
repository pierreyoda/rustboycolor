use super::test_cpu;
// SUB_r_x : substract register X from register A
// we only perform deep testing here since alu_sub is used by ALL instructions
macro_rules! test_SUB_r_X {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.a = 0x02;
                    cpu.regs.$x = 0x05;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0xFD);
                assert_eq!(machine.cpu.regs.f, N_FLAG);
            }
            // {
            //     let machine = test_cpu(&[$instr], |cpu| {
            //         cpu.regs.a = 0x0E;
            //         cpu.regs.$x = 0x08;
            //     });
            //     assert_eq!(machine.clock_cycles(), 4);
            //     assert_eq!(machine.cpu.regs.a, 0x16);
            //     assert_eq!(machine.cpu.regs.f, N_FLAG | H_FLAG);
            // }
            // {
            //     let machine = test_cpu(&[$instr], |cpu| {
            //         cpu.regs.a = 0x80;
            //         cpu.regs.$x = 0x80;
            //     });
            //     assert_eq!(machine.clock_cycles(), 4);
            //     assert_eq!(machine.cpu.regs.a, 0x00);
            //     assert_eq!(machine.cpu.regs.f, N_FLAG | Z_FLAG | C_FLAG);
            // }
            // {
            //     let machine = test_cpu(&[$instr], |cpu| {
            //         cpu.regs.f = N_FLAG;
            //         cpu.regs.a = 0xCC;
            //         cpu.regs.$x = 0x88;
            //     });
            //     assert_eq!(machine.clock_cycles(), 4);
            //     assert_eq!(machine.cpu.regs.a, 0x54);
            //     assert_eq!(machine.cpu.regs.f, N_FLAG | H_FLAG | C_FLAG);
            // }
        }
    )*
    }
}
test_SUB_r_X! {
    test_SUB_r_b: (0x90, b),
    test_SUB_r_c: (0x91, c),
    test_SUB_r_d: (0x92, d),
    test_SUB_r_e: (0x93, e),
    test_SUB_r_h: (0x94, h),
    test_SUB_r_l: (0x95, l),
}
