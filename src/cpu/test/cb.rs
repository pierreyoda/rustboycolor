#![allow(non_snake_case)]

use super::test_cpu;
use memory::Memory;
use registers::{Z_FLAG, N_FLAG, H_FLAG, C_FLAG};

// SWAP_r_X : swap register X's nibbles, reset NHC flags and set Z flag
macro_rules! test_SWAP_r_X {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
                });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.$x, 0);
                assert_eq!(machine.cpu.regs.f, Z_FLAG);
            }
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.f = Z_FLAG |N_FLAG | H_FLAG | C_FLAG;
                    cpu.regs.$x = 0b_1010_1101;
                });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.$x, 0b_1101_1010);
                assert_eq!(machine.cpu.regs.f, 0);
            }
        }
    )*
    }
}
test_SWAP_r_X! {
    test_CB_SWAP_r_b: (0x30, b),
    test_CB_SWAP_r_c: (0x31, c),
    test_CB_SWAP_r_d: (0x32, d),
    test_CB_SWAP_r_e: (0x33, e),
    test_CB_SWAP_r_h: (0x34, h),
    test_CB_SWAP_r_l: (0x35, l),
    test_CB_SWAP_r_a: (0x37, a),
}

// SWAP_HLm : swap (HL)'s nibbles, reset NHC flags and set Z flag
#[test]
fn test_CB_SWAP_HLm() {
    {
        let mut machine = test_cpu(&[0xCB, 0x36], |cpu| {
            cpu.regs.f = N_FLAG | H_FLAG | C_FLAG;
            cpu.regs.set_hl(0x29DA);
        });
        assert_eq!(machine.clock_cycles(), 16);
        assert_eq!(machine.cpu.mem.read_byte(0x29DA), 0);
        assert_eq!(machine.cpu.regs.f, Z_FLAG);
    }
    {
        let mut machine = test_cpu(&[0xCB, 0x36], |cpu| {
            cpu.regs.f = Z_FLAG | N_FLAG | H_FLAG | C_FLAG;
            cpu.regs.set_hl(0x29DA);
            cpu.mem.write_byte(cpu.regs.hl(), 0b_0110_1100);
        });
        assert_eq!(machine.clock_cycles(), 16);
        assert_eq!(machine.cpu.mem.read_byte(0x29DA), 0b_1100_0110);
        assert_eq!(machine.cpu.regs.f, 0);
    }
}

// BIT b, X : set the Z flag against the byte of index b in register X
// also set the H flag to 1 and the N flag to 0
macro_rules! test_BIT_b_r_X {
    ($ ( $name: ident : ($instr: expr, $bit: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.set_flag(N_FLAG, true);
                    cpu.regs.$x = 1 << $bit;
                });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.f, H_FLAG);
            }
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.set_flag(N_FLAG, true);
                    cpu.regs.$x = 0;
                });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.f, Z_FLAG | H_FLAG);
            }
        }
    )*
    }
}
test_BIT_b_r_X! {
    // B
    test_CB_BIT_0_r_b: (0x40, 0, b),
    test_CB_BIT_1_r_b: (0x48, 1, b),
    test_CB_BIT_2_r_b: (0x50, 2, b),
    test_CB_BIT_3_r_b: (0x58, 3, b),
    test_CB_BIT_4_r_b: (0x60, 4, b),
    test_CB_BIT_5_r_b: (0x68, 5, b),
    test_CB_BIT_6_r_b: (0x70, 6, b),
    test_CB_BIT_7_r_b: (0x78, 7, b),
    // C
    test_CB_BIT_0_r_c: (0x41, 0, c),
    test_CB_BIT_1_r_c: (0x49, 1, c),
    test_CB_BIT_2_r_c: (0x51, 2, c),
    test_CB_BIT_3_r_c: (0x59, 3, c),
    test_CB_BIT_4_r_c: (0x61, 4, c),
    test_CB_BIT_5_r_c: (0x69, 5, c),
    test_CB_BIT_6_r_c: (0x71, 6, c),
    test_CB_BIT_7_r_c: (0x79, 7, c),
    // D
    test_CB_BIT_0_r_d: (0x42, 0, d),
    test_CB_BIT_1_r_d: (0x4A, 1, d),
    test_CB_BIT_2_r_d: (0x52, 2, d),
    test_CB_BIT_3_r_d: (0x5A, 3, d),
    test_CB_BIT_4_r_d: (0x62, 4, d),
    test_CB_BIT_5_r_d: (0x6A, 5, d),
    test_CB_BIT_6_r_d: (0x72, 6, d),
    test_CB_BIT_7_r_d: (0x7A, 7, d),
    // E
    test_CB_BIT_0_r_e: (0x43, 0, e),
    test_CB_BIT_1_r_e: (0x4B, 1, e),
    test_CB_BIT_2_r_e: (0x53, 2, e),
    test_CB_BIT_3_r_e: (0x5B, 3, e),
    test_CB_BIT_4_r_e: (0x63, 4, e),
    test_CB_BIT_5_r_e: (0x6B, 5, e),
    test_CB_BIT_6_r_e: (0x73, 6, e),
    test_CB_BIT_7_r_e: (0x7B, 7, e),
    // H
    test_CB_BIT_0_r_h: (0x44, 0, h),
    test_CB_BIT_1_r_h: (0x4C, 1, h),
    test_CB_BIT_2_r_h: (0x54, 2, h),
    test_CB_BIT_3_r_h: (0x5C, 3, h),
    test_CB_BIT_4_r_h: (0x64, 4, h),
    test_CB_BIT_5_r_h: (0x6C, 5, h),
    test_CB_BIT_6_r_h: (0x74, 6, h),
    test_CB_BIT_7_r_h: (0x7C, 7, h),
    // L
    test_CB_BIT_0_r_l: (0x45, 0, l),
    test_CB_BIT_1_r_l: (0x4D, 1, l),
    test_CB_BIT_2_r_l: (0x55, 2, l),
    test_CB_BIT_3_r_l: (0x5D, 3, l),
    test_CB_BIT_4_r_l: (0x65, 4, l),
    test_CB_BIT_5_r_l: (0x6D, 5, l),
    test_CB_BIT_6_r_l: (0x75, 6, l),
    test_CB_BIT_7_r_l: (0x7D, 7, l),
    // A
    test_CB_BIT_0_r_a: (0x47, 0, a),
    test_CB_BIT_1_r_a: (0x4F, 1, a),
    test_CB_BIT_2_r_a: (0x57, 2, a),
    test_CB_BIT_3_r_a: (0x5F, 3, a),
    test_CB_BIT_4_r_a: (0x67, 4, a),
    test_CB_BIT_5_r_a: (0x6F, 5, a),
    test_CB_BIT_6_r_a: (0x77, 6, a),
    test_CB_BIT_7_r_a: (0x7F, 7, a),
}

// BIT b, X : set the Z flag against the byte of index b in (HL)
// also set the H flag to 1 and the N flag to 0
macro_rules! test_BIT_b_HLm {
    ($ ( $name: ident : ($instr: expr, $bit: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.set_flag(N_FLAG, true);
                    cpu.regs.set_hl(0x3B7F);
                    cpu.mem.write_byte(cpu.regs.hl(), 1 << $bit);
                });
                assert_eq!(machine.clock_cycles(), 12);
                assert_eq!(machine.cpu.regs.f, H_FLAG);
            }
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.set_flag(N_FLAG, true);
                    cpu.regs.set_hl(0x3B7F);
                    cpu.mem.write_byte(cpu.regs.hl(), 0);
                });
                assert_eq!(machine.clock_cycles(), 12);
                assert_eq!(machine.cpu.regs.f, Z_FLAG | H_FLAG);
            }
        }
    )*
    }
}
test_BIT_b_HLm! {
    test_CB_BIT_0_HLm: (0x46, 0),
    test_CB_BIT_1_HLm: (0x4E, 1),
    test_CB_BIT_2_HLm: (0x56, 2),
    test_CB_BIT_3_HLm: (0x5E, 3),
    test_CB_BIT_4_HLm: (0x66, 4),
    test_CB_BIT_5_HLm: (0x6E, 5),
    test_CB_BIT_6_HLm: (0x76, 6),
    test_CB_BIT_7_HLm: (0x7E, 7),
}

// RES b, X : set to 0 the byte of index b in register X
macro_rules! test_RES_b_r_X {
    ($ ( $name: ident : ($instr: expr, $bit: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| { cpu.regs.$x = 1 << $bit; });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.$x, 0);
            }
            {
                let machine = test_cpu(&[0xCB, $instr], |cpu| { cpu.regs.$x = 0xFF; });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.$x, 0xFF ^ (1 << $bit));
            }
        }
    )*
    }
}
test_RES_b_r_X! {
    // B
    test_CB_RES_0_r_b: (0x80, 0, b),
    test_CB_RES_1_r_b: (0x88, 1, b),
    test_CB_RES_2_r_b: (0x90, 2, b),
    test_CB_RES_3_r_b: (0x98, 3, b),
    test_CB_RES_4_r_b: (0xA0, 4, b),
    test_CB_RES_5_r_b: (0xA8, 5, b),
    test_CB_RES_6_r_b: (0xB0, 6, b),
    test_CB_RES_7_r_b: (0xB8, 7, b),
    // C
    test_CB_RES_0_r_c: (0x81, 0, c),
    test_CB_RES_1_r_c: (0x89, 1, c),
    test_CB_RES_2_r_c: (0x91, 2, c),
    test_CB_RES_3_r_c: (0x99, 3, c),
    test_CB_RES_4_r_c: (0xA1, 4, c),
    test_CB_RES_5_r_c: (0xA9, 5, c),
    test_CB_RES_6_r_c: (0xB1, 6, c),
    test_CB_RES_7_r_c: (0xB9, 7, c),
    // D
    test_CB_RES_0_r_d: (0x82, 0, d),
    test_CB_RES_1_r_d: (0x8A, 1, d),
    test_CB_RES_2_r_d: (0x92, 2, d),
    test_CB_RES_3_r_d: (0x9A, 3, d),
    test_CB_RES_4_r_d: (0xA2, 4, d),
    test_CB_RES_5_r_d: (0xAA, 5, d),
    test_CB_RES_6_r_d: (0xB2, 6, d),
    test_CB_RES_7_r_d: (0xBA, 7, d),
    // E
    test_CB_RES_0_r_e: (0x83, 0, e),
    test_CB_RES_1_r_e: (0x8B, 1, e),
    test_CB_RES_2_r_e: (0x93, 2, e),
    test_CB_RES_3_r_e: (0x9B, 3, e),
    test_CB_RES_4_r_e: (0xA3, 4, e),
    test_CB_RES_5_r_e: (0xAB, 5, e),
    test_CB_RES_6_r_e: (0xB3, 6, e),
    test_CB_RES_7_r_e: (0xBB, 7, e),
    // H
    test_CB_RES_0_r_h: (0x84, 0, h),
    test_CB_RES_1_r_h: (0x8C, 1, h),
    test_CB_RES_2_r_h: (0x94, 2, h),
    test_CB_RES_3_r_h: (0x9C, 3, h),
    test_CB_RES_4_r_h: (0xA4, 4, h),
    test_CB_RES_5_r_h: (0xAC, 5, h),
    test_CB_RES_6_r_h: (0xB4, 6, h),
    test_CB_RES_7_r_h: (0xBC, 7, h),
    // L
    test_CB_RES_0_r_l: (0x85, 0, l),
    test_CB_RES_1_r_l: (0x8D, 1, l),
    test_CB_RES_2_r_l: (0x95, 2, l),
    test_CB_RES_3_r_l: (0x9D, 3, l),
    test_CB_RES_4_r_l: (0xA5, 4, l),
    test_CB_RES_5_r_l: (0xAD, 5, l),
    test_CB_RES_6_r_l: (0xB5, 6, l),
    test_CB_RES_7_r_l: (0xBD, 7, l),
    // A
    test_CB_RES_0_r_a: (0x87, 0, a),
    test_CB_RES_1_r_a: (0x8F, 1, a),
    test_CB_RES_2_r_a: (0x97, 2, a),
    test_CB_RES_3_r_a: (0x9F, 3, a),
    test_CB_RES_4_r_a: (0xA7, 4, a),
    test_CB_RES_5_r_a: (0xAF, 5, a),
    test_CB_RES_6_r_a: (0xB7, 6, a),
    test_CB_RES_7_r_a: (0xBF, 7, a),
}

// RES b, (HL) : set to 0 the byte of index b in (HL)
macro_rules! test_RES_b_HLm {
    ($ ( $name: ident : ($instr: expr, $bit: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let mut machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.set_hl(0xB9E2);
                    cpu.mem.write_byte(cpu.regs.hl(), 1 << $bit);
                });
                assert_eq!(machine.clock_cycles(), 16);
                assert_eq!(machine.cpu.mem.read_byte(0xB9E2), 0);
            }
            {
                let mut machine = test_cpu(&[0xCB, $instr], |cpu| {
                    cpu.regs.set_hl(0xB9E2);
                    cpu.mem.write_byte(cpu.regs.hl(), 0xFF);
                });
                assert_eq!(machine.clock_cycles(), 16);
                assert_eq!(machine.cpu.mem.read_byte(0xB9E2), 0xFF ^ (1 << $bit));
            }
        }
    )*
    }
}
test_RES_b_HLm! {
    test_CB_RES_0_HLm: (0x86, 0),
    test_CB_RES_1_HLm: (0x8E, 1),
    test_CB_RES_2_HLm: (0x96, 2),
    test_CB_RES_3_HLm: (0x9E, 3),
    test_CB_RES_4_HLm: (0xA6, 4),
    test_CB_RES_5_HLm: (0xAE, 5),
    test_CB_RES_6_HLm: (0xB6, 6),
    test_CB_RES_7_HLm: (0xBE, 7),
}

// SET b, X : set to 1 the byte of index b in register X
macro_rules! test_SET_b_r_X {
    ($ ( $name: ident : ($instr: expr, $bit: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            let machine = test_cpu(&[0xCB, $instr], |_| {});
            assert_eq!(machine.clock_cycles(), 8);
            assert_eq!(machine.cpu.regs.$x, 1 << $bit);
        }
    )*
    }
}
test_SET_b_r_X! {
    // B
    test_CB_SET_0_r_b: (0xC0, 0, b),
    test_CB_SET_1_r_b: (0xC8, 1, b),
    test_CB_SET_2_r_b: (0xD0, 2, b),
    test_CB_SET_3_r_b: (0xD8, 3, b),
    test_CB_SET_4_r_b: (0xE0, 4, b),
    test_CB_SET_5_r_b: (0xE8, 5, b),
    test_CB_SET_6_r_b: (0xF0, 6, b),
    test_CB_SET_7_r_b: (0xF8, 7, b),
    // C
    test_CB_SET_0_r_c: (0xC1, 0, c),
    test_CB_SET_1_r_c: (0xC9, 1, c),
    test_CB_SET_2_r_c: (0xD1, 2, c),
    test_CB_SET_3_r_c: (0xD9, 3, c),
    test_CB_SET_4_r_c: (0xE1, 4, c),
    test_CB_SET_5_r_c: (0xE9, 5, c),
    test_CB_SET_6_r_c: (0xF1, 6, c),
    test_CB_SET_7_r_c: (0xF9, 7, c),
    // D
    test_CB_SET_0_r_d: (0xC2, 0, d),
    test_CB_SET_1_r_d: (0xCA, 1, d),
    test_CB_SET_2_r_d: (0xD2, 2, d),
    test_CB_SET_3_r_d: (0xDA, 3, d),
    test_CB_SET_4_r_d: (0xE2, 4, d),
    test_CB_SET_5_r_d: (0xEA, 5, d),
    test_CB_SET_6_r_d: (0xF2, 6, d),
    test_CB_SET_7_r_d: (0xFA, 7, d),
    // E
    test_CB_SET_0_r_e: (0xC3, 0, e),
    test_CB_SET_1_r_e: (0xCB, 1, e),
    test_CB_SET_2_r_e: (0xD3, 2, e),
    test_CB_SET_3_r_e: (0xDB, 3, e),
    test_CB_SET_4_r_e: (0xE3, 4, e),
    test_CB_SET_5_r_e: (0xEB, 5, e),
    test_CB_SET_6_r_e: (0xF3, 6, e),
    test_CB_SET_7_r_e: (0xFB, 7, e),
    // H
    test_CB_SET_0_r_h: (0xC4, 0, h),
    test_CB_SET_1_r_h: (0xCC, 1, h),
    test_CB_SET_2_r_h: (0xD4, 2, h),
    test_CB_SET_3_r_h: (0xDC, 3, h),
    test_CB_SET_4_r_h: (0xE4, 4, h),
    test_CB_SET_5_r_h: (0xEC, 5, h),
    test_CB_SET_6_r_h: (0xF4, 6, h),
    test_CB_SET_7_r_h: (0xFC, 7, h),
    // L
    test_CB_SET_0_r_l: (0xC5, 0, l),
    test_CB_SET_1_r_l: (0xCD, 1, l),
    test_CB_SET_2_r_l: (0xD5, 2, l),
    test_CB_SET_3_r_l: (0xDD, 3, l),
    test_CB_SET_4_r_l: (0xE5, 4, l),
    test_CB_SET_5_r_l: (0xED, 5, l),
    test_CB_SET_6_r_l: (0xF5, 6, l),
    test_CB_SET_7_r_l: (0xFD, 7, l),
    // A
    test_CB_SET_0_r_a: (0xC7, 0, a),
    test_CB_SET_1_r_a: (0xCF, 1, a),
    test_CB_SET_2_r_a: (0xD7, 2, a),
    test_CB_SET_3_r_a: (0xDF, 3, a),
    test_CB_SET_4_r_a: (0xE7, 4, a),
    test_CB_SET_5_r_a: (0xEF, 5, a),
    test_CB_SET_6_r_a: (0xF7, 6, a),
    test_CB_SET_7_r_a: (0xFF, 7, a),
}

// SET b, (HL) : set to 1 the byte of index b in (HL)
macro_rules! test_SET_b_HLm {
    ($ ( $name: ident : ($instr: expr, $bit: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            let mut machine = test_cpu(&[0xCB, $instr], |cpu| { cpu.regs.set_hl(0xCF3A); });
            assert_eq!(machine.clock_cycles(), 16);
            assert_eq!(machine.cpu.mem.read_byte(0xCF3A), 1 << $bit);
        }
    )*
    }
}
test_SET_b_HLm! {
    test_CB_SET_0_HLm: (0xC6, 0),
    test_CB_SET_1_HLm: (0xCE, 1),
    test_CB_SET_2_HLm: (0xD6, 2),
    test_CB_SET_3_HLm: (0xDE, 3),
    test_CB_SET_4_HLm: (0xE6, 4),
    test_CB_SET_5_HLm: (0xEE, 5),
    test_CB_SET_6_HLm: (0xF6, 6),
    test_CB_SET_7_HLm: (0xFE, 7),
}
