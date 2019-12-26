#![allow(non_snake_case)]

use super::{test_cpu, OPCODE_END};
use crate::memory::Memory;
use crate::registers::{Z_FLAG, C_FLAG};

// JP_nn : absolute jump to 16-bit address
#[test]
fn test_JP_nn() {
    let machine = test_cpu(&[0xC3, 0xA9, 0x5D], |cpu| {
        cpu.mem.write_byte(0x5DA9, OPCODE_END);
    });
    assert_eq!(machine.clock_cycles(), 16);
    assert_eq!(machine.cpu.regs.pc, 0x5DA9);
}

// JP_HLm : absolute jump to (HL)
#[test]
fn test_JP_HLm() {
    let machine = test_cpu(&[0xE9], |cpu| {
        cpu.regs.set_hl(0xBF5C);
        cpu.mem.write_byte(cpu.regs.hl(), OPCODE_END);
    });
    assert_eq!(machine.clock_cycles(), 4);
    assert_eq!(machine.cpu.regs.pc, 0xBF5C);
}

// JP_cond_nn : absolute jump to 16-bit address if a condition is met
macro_rules! test_JP_cond_nn {
    ($ ( $name: ident : ($instr: expr, $f1: expr, $f2: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr, 0xCB, 0x3F], |cpu| {
                    cpu.regs.f = $f1;
                    cpu.mem.write_byte(0x3FCB, OPCODE_END);
                });
                assert_eq!(machine.clock_cycles(), 16);
                assert_eq!(machine.cpu.regs.pc, 0x3FCB);
            }
            {
                let machine = test_cpu(&[$instr, 0xCB, 0x3F], |cpu| { cpu.regs.f = $f2 });
                assert_eq!(machine.clock_cycles(), 12);
                assert_eq!(machine.cpu.regs.pc, 0x03);
            }
        }
    )*
    }
}
test_JP_cond_nn! {
    test_JP_NZ_nn: (0xC2, 0, Z_FLAG),
    test_JP_NC_nn: (0xD2, 0, C_FLAG),
    test_JP_Z_nn: (0xCA, Z_FLAG, 0),
    test_JP_C_nn: (0xDA, C_FLAG, 0),
}

// JR_n : relative jump by signed immediate byte
#[test]
fn test_JR_n() {
    {
        let machine = test_cpu(&[0x18, 0x03], |cpu| {
            cpu.mem.write_byte(0x05, OPCODE_END);
        });
        assert_eq!(machine.clock_cycles(), 12);
        assert_eq!(machine.cpu.regs.pc, 0x05);
    }
    {
        let machine = test_cpu(&[OPCODE_END, 0x00, 0x18, 0xFC], |cpu| { cpu.regs.pc = 0x02 });
        assert_eq!(machine.clock_cycles(), 12);
        assert_eq!(machine.cpu.regs.pc, 0x00);
    }
}

// JP_cond_nn : relative jump by immediate byte if a condition is met
macro_rules! test_JP_cond_n {
    ($ ( $name: ident : ($instr: expr, $f1: expr, $f2: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr, 0x03], |cpu| {
                    cpu.regs.f = $f1;
                    cpu.mem.write_byte(0x05, OPCODE_END);
                });
                assert_eq!(machine.clock_cycles(), 12);
                assert_eq!(machine.cpu.regs.pc, 0x05);
            }
            {
                let machine = test_cpu(&[OPCODE_END, 0x00, $instr, 0xFC], |cpu| {
                    cpu.regs.f = $f1;
                    cpu.regs.pc = 0x02
                });
                assert_eq!(machine.clock_cycles(), 12);
                assert_eq!(machine.cpu.regs.pc, 0x00);
            }
            {
                let machine = test_cpu(&[$instr, 0xFC], |cpu| { cpu.regs.f = $f2 });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.pc, 0x02);
            }
        }
    )*
    }
}
test_JP_cond_n! {
    test_JP_NZ_n: (0x20, 0, Z_FLAG),
    test_JP_NC_n: (0x30, 0, C_FLAG),
    test_JP_Z_n: (0x28, Z_FLAG, 0),
    test_JR_C_n: (0x38, C_FLAG, 0),
}

// CALL_nn : call routine at 16-bit address
#[test]
fn test_CALL_nn() {
    let mut machine = test_cpu(&[], |cpu| {
        cpu.regs.sp = 0xFFFE;
        cpu.regs.pc = 0xA3E9;
        cpu.mem.write_byte(cpu.regs.pc, 0xCD); // CALL nn
        cpu.mem.write_byte(cpu.regs.pc+1, 0x78);
        cpu.mem.write_byte(cpu.regs.pc+2, 0xDF);
        cpu.mem.write_byte(0xDF78, OPCODE_END);
    });
    assert_eq!(machine.clock_cycles(), 24);
    assert_eq!(machine.cpu.regs.pc, 0xDF78);
    assert_eq!(machine.cpu.regs.sp, 0xFFFC);
    // regs.pc stored = 0xA3EC
    assert_eq!(machine.cpu.mem.read_byte(0xFFFC), 0xEC);
    assert_eq!(machine.cpu.mem.read_byte(0xFFFD), 0xA3);
}

// CALL_cond_nn : call routine at 16-bit address if a condition is met
macro_rules! test_CALL_cond_n {
    ($ ( $name: ident : ($instr: expr, $f1: expr, $f2: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let mut machine = test_cpu(&[], |cpu| {
                    cpu.regs.f = $f1;
                    cpu.regs.sp = 0xFFFE;
                    cpu.regs.pc = 0xA3E9;
                    cpu.mem.write_byte(cpu.regs.pc, $instr);
                    cpu.mem.write_byte(cpu.regs.pc+1, 0x78);
                    cpu.mem.write_byte(cpu.regs.pc+2, 0xDF);
                    cpu.mem.write_byte(0xDF78, OPCODE_END);
                });
                assert_eq!(machine.clock_cycles(), 24);
                assert_eq!(machine.cpu.regs.pc, 0xDF78);
                assert_eq!(machine.cpu.regs.sp, 0xFFFC);
                assert_eq!(machine.cpu.mem.read_byte(0xFFFC), 0xEC);
                assert_eq!(machine.cpu.mem.read_byte(0xFFFD), 0xA3);
            }
            {
                let machine = test_cpu(&[$instr, 0x78, 0xDF], |cpu| { cpu.regs.f = $f2 });
                assert_eq!(machine.clock_cycles(), 12);
                assert_eq!(machine.cpu.regs.pc, 0x03);
            }
        }
    )*
    }
}
test_CALL_cond_n! {
    test_CALL_NZ_n: (0xC4, 0, Z_FLAG),
    test_CALL_NC_n: (0xD4, 0, C_FLAG),
    test_CALL_Z_n: (0xCC, Z_FLAG, 0),
    test_CALL_C_n: (0xDC, C_FLAG, 0),
}

// RET : return to calling routine
#[test]
fn test_RET() {
    let machine = test_cpu(&[0xC9], |cpu| {
        cpu.regs.sp = 0xFFFC;
        cpu.mem.write_byte(0xFFFC, 0xEC);
        cpu.mem.write_byte(0xFFFD, 0xA3);
        cpu.mem.write_byte(0xA3EC, OPCODE_END);
    });
    assert_eq!(machine.clock_cycles(), 16);
    assert_eq!(machine.cpu.regs.sp, 0xFFFE);
    assert_eq!(machine.cpu.regs.pc, 0xA3EC);
}

// RETI : enable interrupts and return to calling routine
#[test]
fn test_RETI() {
    let machine = test_cpu(&[0xD9], |cpu| {
        cpu.ime = false;
        cpu.regs.sp = 0xFFFC;
        cpu.mem.write_byte(0xFFFC, 0xEC);
        cpu.mem.write_byte(0xFFFD, 0xA3);
        cpu.mem.write_byte(0xA3EC, OPCODE_END);
    });
    assert_eq!(machine.clock_cycles(), 16);
    assert_eq!(machine.cpu.regs.sp, 0xFFFE);
    assert_eq!(machine.cpu.regs.pc, 0xA3EC);
    assert_eq!(machine.cpu.ime, true);
}

// RET_cond : return to calling routine if a condition is met
macro_rules! test_RET_cond_n {
    ($ ( $name: ident : ($instr: expr, $f1: expr, $f2: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            {
                let machine = test_cpu(&[$instr], |cpu| {
                    cpu.regs.f = $f1;
                    cpu.regs.sp = 0xFFFC;
                    cpu.mem.write_byte(0xFFFC, 0xEC);
                    cpu.mem.write_byte(0xFFFD, 0xA3);
                    cpu.mem.write_byte(0xA3EC, OPCODE_END);
                });
                assert_eq!(machine.clock_cycles(), 20);
                assert_eq!(machine.cpu.regs.sp, 0xFFFE);
                assert_eq!(machine.cpu.regs.pc, 0xA3EC);
            }
            {
                let machine = test_cpu(&[$instr], |cpu| { cpu.regs.f = $f2 });
                assert_eq!(machine.clock_cycles(), 8);
                assert_eq!(machine.cpu.regs.pc, 0x01);
            }
        }
    )*
    }
}
test_RET_cond_n! {
    test_RET_NZ_n: (0xC0, 0, Z_FLAG),
    test_RET_NC_n: (0xD0, 0, C_FLAG),
    test_RET_Z_n: (0xC8, Z_FLAG, 0),
    test_RET_C_n: (0xD8, C_FLAG, 0),
}

// RST_xxH : call routine at address 0x00XX
macro_rules! test_RST {
    ($ ( $name: ident : ($instr: expr, $address: expr), )* ) => {
    $(
        #[test]
        fn $name() {
            let mut machine = test_cpu(&[], |cpu| {
                cpu.regs.sp = 0xFFFE;
                cpu.regs.pc = 0xA3E9;
                cpu.mem.write_byte(cpu.regs.pc, $instr);
                cpu.mem.write_byte($address, OPCODE_END);
            });
            assert_eq!(machine.clock_cycles(), 16);
            assert_eq!(machine.cpu.regs.pc, $address);
            assert_eq!(machine.cpu.regs.sp, 0xFFFC);
            assert_eq!(machine.cpu.mem.read_byte(0xFFFC), 0xEA);
            assert_eq!(machine.cpu.mem.read_byte(0xFFFD), 0xA3);
        }
    )*
    }
}
test_RST! {
    test_RST_00H: (0xC7, 0x0000),
    test_RST_08H: (0xCF, 0x0008),
    test_RST_10H: (0xD7, 0x0010),
    test_RST_18H: (0xDF, 0x0018),
    test_RST_20H: (0xE7, 0x0020),
    test_RST_28H: (0xEF, 0x0028),
    test_RST_30H: (0xF7, 0x0030),
    test_RST_38H: (0xFF, 0x0038),
}
