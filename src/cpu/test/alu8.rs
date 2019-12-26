#![allow(non_snake_case)]

use super::test_cpu;
use crate::memory::Memory;
use crate::registers::{C_FLAG, H_FLAG, N_FLAG, Z_FLAG};

// ADD_r_x : add register X to register A
// we only perform deep testing here since alu_add is used by ALL instructions
macro_rules! test_ADD_r_X {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG;
                    cpu.regs.a = 0x02;
                    cpu.regs.$x = 0x05;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0x7);
                assert_eq!(machine.cpu.regs.f, 0);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG;
                    cpu.regs.a = 0x0E;
                    cpu.regs.$x = 0x08;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0x16);
                assert_eq!(machine.cpu.regs.f, H_FLAG);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG;
                    cpu.regs.a = 0x80;
                    cpu.regs.$x = 0x80;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0x00);
                assert_eq!(machine.cpu.regs.f, Z_FLAG | C_FLAG);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG;
                    cpu.regs.a = 0xCC;
                    cpu.regs.$x = 0x88;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0x54);
                assert_eq!(machine.cpu.regs.f, H_FLAG | C_FLAG);
            }
        }
    )*
    }
}
test_ADD_r_X! {
    test_ADD_r_b: (0x80, b),
    test_ADD_r_c: (0x81, c),
    test_ADD_r_d: (0x82, d),
    test_ADD_r_e: (0x83, e),
    test_ADD_r_h: (0x84, h),
    test_ADD_r_l: (0x85, l),
}

// AND_r_x : logical AND register X against register A
macro_rules! test_AND_r_X {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG | C_FLAG;
                    cpu.regs.a  = 0b_0101_1101;
                    cpu.regs.$x = 0b_1100_0111;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0b_0100_0101);
                assert_eq!(machine.cpu.regs.f, H_FLAG);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG | C_FLAG;
                    cpu.regs.a  = 0b_0101_1101;
                    cpu.regs.$x = 0;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0);
                assert_eq!(machine.cpu.regs.f, Z_FLAG | H_FLAG);
            }
        }
    )*
    }
}
test_AND_r_X! {
    test_AND_r_b: (0xA0, b),
    test_AND_r_c: (0xA1, c),
    test_AND_r_d: (0xA2, d),
    test_AND_r_e: (0xA3, e),
    test_AND_r_h: (0xA4, h),
    test_AND_r_l: (0xA5, l),
}

// AND_HLm : logical AND (HL) against register A
#[test]
fn test_AND_HLm() {
    {
        let machine = test_cpu(&[0xA6], |cpu| {
            cpu.regs.f = N_FLAG | C_FLAG;
            cpu.regs.a = 0b_0101_1101;
            cpu.regs.set_hl(0xAF3D);
            cpu.mem.write_byte(cpu.regs.hl(), 0b_1100_0111);
        });
        assert_eq!(machine.clock_cycles(), 8);
        assert_eq!(machine.cpu.regs.a, 0b_0100_0101);
        assert_eq!(machine.cpu.regs.f, H_FLAG);
    }
    {
        let machine = test_cpu(&[0xA6], |cpu| {
            cpu.regs.f = N_FLAG | C_FLAG;
            cpu.regs.a = 0;
            cpu.regs.set_hl(0xAF3D);
            cpu.mem.write_byte(cpu.regs.hl(), 0);
        });
        assert_eq!(machine.clock_cycles(), 8);
        assert_eq!(machine.cpu.regs.a, 0);
        assert_eq!(machine.cpu.regs.f, Z_FLAG | H_FLAG);
    }
}

// OR_r_x : logical OR register X against register A
macro_rules! test_OR_r_X {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
                    cpu.regs.a  = 0b_0101_1101;
                    cpu.regs.$x = 0b_1100_0111;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0b_1101_1111);
                assert_eq!(machine.cpu.regs.f, 0);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
                    cpu.regs.a  = 0;
                    cpu.regs.$x = 0;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0);
                assert_eq!(machine.cpu.regs.f, Z_FLAG);
            }
        }
    )*
    }
}
test_OR_r_X! {
    test_OR_r_b: (0xB0, b),
    test_OR_r_c: (0xB1, c),
    test_OR_r_d: (0xB2, d),
    test_OR_r_e: (0xB3, e),
    test_OR_r_h: (0xB4, h),
    test_OR_r_l: (0xB5, l),
}

// OR_HLm : logical OR (HL) against register A
#[test]
fn test_OR_HLm() {
    {
        let machine = test_cpu(&[0xB6], |cpu| {
            cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
            cpu.regs.a = 0b_0101_1101;
            cpu.regs.set_hl(0xAF3D);
            cpu.mem.write_byte(cpu.regs.hl(), 0b_1100_0111);
        });
        assert_eq!(machine.clock_cycles(), 8);
        assert_eq!(machine.cpu.regs.a, 0b_1101_1111);
        assert_eq!(machine.cpu.regs.f, 0);
    }
    {
        let machine = test_cpu(&[0xB6], |cpu| {
            cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
            cpu.regs.a = 0;
            cpu.regs.set_hl(0xAF3D);
            cpu.mem.write_byte(cpu.regs.hl(), 0);
        });
        assert_eq!(machine.clock_cycles(), 8);
        assert_eq!(machine.cpu.regs.a, 0);
        assert_eq!(machine.cpu.regs.f, Z_FLAG);
    }
}

// XOR_r_x : logical XOR register X against register A
macro_rules! test_XOR_r_X {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
                    cpu.regs.a  = 0b_0101_1101;
                    cpu.regs.$x = 0b_1100_0111;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0b_1001_1010);
                assert_eq!(machine.cpu.regs.f, 0);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
                    cpu.regs.a  = 0b_0101_1101;
                    cpu.regs.$x = cpu.regs.a;
                });
                assert_eq!(machine.clock_cycles(), 4);
                assert_eq!(machine.cpu.regs.a, 0);
                assert_eq!(machine.cpu.regs.f, Z_FLAG);
            }
        }
    )*
    }
}
test_XOR_r_X! {
    test_XOR_r_b: (0xA8, b),
    test_XOR_r_c: (0xA9, c),
    test_XOR_r_d: (0xAA, d),
    test_XOR_r_e: (0xAB, e),
    test_XOR_r_h: (0xAC, h),
    test_XOR_r_l: (0xAD, l),
}

// XOR_HLm : logical XOR (HL) against register A
#[test]
fn test_XOR_HLm() {
    {
        let machine = test_cpu(&[0xAE], |cpu| {
            cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
            cpu.regs.a = 0b_0101_1101;
            cpu.regs.set_hl(0xAF3D);
            cpu.mem.write_byte(cpu.regs.hl(), 0b_1100_0111);
        });
        assert_eq!(machine.clock_cycles(), 8);
        assert_eq!(machine.cpu.regs.a, 0b_1001_1010);
        assert_eq!(machine.cpu.regs.f, 0);
    }
    {
        let machine = test_cpu(&[0xAE], |cpu| {
            cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
            cpu.regs.a = 0b_0101_1101;
            cpu.regs.set_hl(0xAF3D);
            cpu.mem.write_byte(cpu.regs.hl(), cpu.regs.a);
        });
        assert_eq!(machine.clock_cycles(), 8);
        assert_eq!(machine.cpu.regs.a, 0);
        assert_eq!(machine.cpu.regs.f, Z_FLAG);
    }
}
