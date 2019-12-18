#![allow(non_snake_case)]

use super::test_cpu;
use memory::Memory;

// LD_rr_xy : load register y in register x
macro_rules! test_LD_rr_xy {
    ($ ( $name: ident : ($instr: expr, $x: ident, $y: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            let machine = test_cpu(&[$instr], |cpu| { cpu.regs.$y = 0x9A; });
            assert_eq!(machine.clock_cycles(), 4);
            assert_eq!(machine.cpu.regs.$x, 0x9A);
        }
    )*
    }
}
test_LD_rr_xy! {
    // bY
    test_LD_rr_bb: (0x40, b, b),
    test_LD_rr_bc: (0x41, b, c),
    test_LD_rr_bd: (0x42, b, d),
    test_LD_rr_be: (0x43, b, e),
    test_LD_rr_bh: (0x44, b, h),
    test_LD_rr_bl: (0x45, b, l),
    test_LD_rr_ba: (0x47, b, a),
    // cY
    test_LD_rr_cb: (0x48, c, b),
    test_LD_rr_cc: (0x49, c, c),
    test_LD_rr_cd: (0x4A, c, d),
    test_LD_rr_ce: (0x4B, c, e),
    test_LD_rr_ch: (0x4C, c, h),
    test_LD_rr_cl: (0x4D, c, l),
    test_LD_rr_ca: (0x4F, c, a),
    // dY
    test_LD_rr_db: (0x50, d, b),
    test_LD_rr_dc: (0x51, d, c),
    test_LD_rr_dd: (0x52, d, d),
    test_LD_rr_de: (0x53, d, e),
    test_LD_rr_dh: (0x54, d, h),
    test_LD_rr_dl: (0x55, d, l),
    test_LD_rr_da: (0x57, d, a),
    // eY
    test_LD_rr_eb: (0x58, e, b),
    test_LD_rr_ec: (0x59, e, c),
    test_LD_rr_ed: (0x5A, e, d),
    test_LD_rr_ee: (0x5B, e, e),
    test_LD_rr_eh: (0x5C, e, h),
    test_LD_rr_el: (0x5D, e, l),
    test_LD_rr_ea: (0x5F, e, a),
    // hY
    test_LD_rr_hb: (0x60, h, b),
    test_LD_rr_hc: (0x61, h, c),
    test_LD_rr_hd: (0x62, h, d),
    test_LD_rr_he: (0x63, h, e),
    test_LD_rr_hh: (0x64, h, h),
    test_LD_rr_hl: (0x65, h, l),
    test_LD_rr_ha: (0x67, h, a),
    // lY
    test_LD_rr_lb: (0x68, l, b),
    test_LD_rr_lc: (0x69, l, c),
    test_LD_rr_ld: (0x6A, l, d),
    test_LD_rr_le: (0x6B, l, e),
    test_LD_rr_lh: (0x6C, l, h),
    test_LD_rr_ll: (0x6D, l, l),
    test_LD_rr_la: (0x6F, l, a),
    // aY
    test_LD_rr_ab: (0x78, a, b),
    test_LD_rr_ac: (0x79, a, c),
    test_LD_rr_ad: (0x7A, a, d),
    test_LD_rr_ae: (0x7B, a, e),
    test_LD_rr_ah: (0x7C, a, h),
    test_LD_rr_al: (0x7D, a, l),
    test_LD_rr_aa: (0x7F, a, a),
}

// LD_r_HLm_x : load the (HL) value in register x
macro_rules! test_LD_r_HLm_x {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            let machine = test_cpu(&[$instr], |cpu| {
                cpu.regs.h = 0x7A;
                cpu.regs.l = 0xD8;
                cpu.mem.write_byte(0x7AD8, 0x9A);
            });
            assert_eq!(machine.clock_cycles(), 8);
            assert_eq!(machine.cpu.regs.$x, 0x9A);
        }
    )*
    }
}
test_LD_r_HLm_x! {
    test_LD_r_HLm_b: (0x46, b),
    test_LD_r_HLm_c: (0x4E, c),
    test_LD_r_HLm_d: (0x56, d),
    test_LD_r_HLm_e: (0x5E, e),
    test_LD_r_HLm_h: (0x66, h),
    test_LD_r_HLm_l: (0x6E, l),
    test_LD_r_HLm_a: (0x7E, a),
}

// LD_HLm_r_x : load the register x value in (HL)
macro_rules! test_LD_HLm_r_x {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            let mut machine = test_cpu(&[$instr], |cpu| {
                cpu.regs.set_hl(0x7AD8);
                cpu.regs.$x = 0x9A;
            });
            assert_eq!(machine.clock_cycles(), 8);
            assert_eq!(machine.cpu.mem.read_byte(0x7AD8), 0x9A);
        }
    )*
    }
}
test_LD_HLm_r_x! {
    test_LD_HLm_r_b: (0x70, b),
    test_LD_HLm_r_c: (0x71, c),
    test_LD_HLm_r_d: (0x72, d),
    test_LD_HLm_r_e: (0x73, e),
    test_LD_HLm_r_a: (0x77, a),
}

#[test]
fn test_LD_HLm_r_h() {
    let mut machine = test_cpu(&[0x74], |cpu| {
        cpu.regs.set_hl(0x7AD8);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0x7AD8), 0x7A);
}
#[test]
fn test_LD_HLm_r_l() {
    let mut machine = test_cpu(&[0x75], |cpu| {
        cpu.regs.set_hl(0x7AD8);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0x7AD8), 0xD8);
}

// LD_r_n_x : load immediate byte into register x
macro_rules! test_LD_r_n_x {
    ($ ( $name: ident : ($instr: expr, $x: ident), )* ) => {
    $(
        #[test]
        fn $name() {
            let machine = test_cpu(&[$instr, 0x9A], |_| {});
            assert_eq!(machine.clock_cycles(), 8);
            assert_eq!(machine.cpu.regs.$x, 0x9A);
        }
    )*
    }
}
test_LD_r_n_x! {
    test_LD_r_n_b: (0x06, b),
    test_LD_r_n_c: (0x0E, c),
    test_LD_r_n_d: (0x16, d),
    test_LD_r_n_e: (0x1E, e),
    test_LD_r_n_h: (0x26, h),
    test_LD_r_n_l: (0x2E, l),
    test_LD_r_n_a: (0x3E, a),
}

// LD_HLm_n : load immediate byte into (HL)
#[test]
fn test_LD_HLm_n() {
    let mut machine = test_cpu(&[0x36, 0x9A], |cpu| {
        cpu.regs.set_hl(0x7AD8);
    });
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.mem.read_byte(0x7AD8), 0x9A);
}

// LD_XYm_A : load A into (rXrY)
#[test]
fn test_LD_BCm_A() {
    let mut machine = test_cpu(&[0x02], |cpu| {
        cpu.regs.set_bc(0x7AD8);
        cpu.regs.a = 0x9A;
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0x7AD8), 0x9A);
}
#[test]
fn test_LD_DEm_A() {
    let mut machine = test_cpu(&[0x12], |cpu| {
        cpu.regs.set_de(0x7AD8);
        cpu.regs.a = 0x9A;
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0x7AD8), 0x9A);
}

// LD_A_XYm : load (rXrY) into A
#[test]
fn test_LD_A_BCm() {
    let machine = test_cpu(&[0x0A], |cpu| {
        cpu.regs.set_bc(0x7AD8);
        cpu.mem.write_byte(cpu.regs.bc(), 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.regs.a, 0x9A);
}
#[test]
fn test_LD_A_DEm() {
    let machine = test_cpu(&[0x1A], |cpu| {
        cpu.regs.set_de(0x7AD8);
        cpu.mem.write_byte(cpu.regs.de(), 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.regs.a, 0x9A);
}

// LD_XY_nn : load immediate word (16 bits) into XY
#[test]
fn test_LD_BC_nn() {
    let machine = test_cpu(&[0x01, 0x4B, 0xDE], |_| {});
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.regs.bc(), 0xDE4B);
}
#[test]
fn test_LD_DE_nn() {
    let machine = test_cpu(&[0x11, 0x4B, 0xDE], |_| {});
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.regs.de(), 0xDE4B);
}
#[test]
fn test_LD_HL_nn() {
    let machine = test_cpu(&[0x21, 0x4B, 0xDE], |_| {});
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.regs.hl(), 0xDE4B);
}
#[test]
fn test_LD_SP_nn() {
    let machine = test_cpu(&[0x31, 0x4B, 0xDE], |_| {});
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.regs.sp, 0xDE4B);
}

// LD_NNm_A : load A into (nn)
#[test]
fn test_LD_NNm_A() {
    let mut machine = test_cpu(&[0xEA, 0x4B, 0xDE], |cpu| {
        cpu.regs.a = 0x9A;
    });
    assert_eq!(machine.clock_cycles(), 16);
    assert_eq!(machine.cpu.mem.read_byte(0xDE4B), 0x9A);
}
// LD_A_NNm : load (nn) into A
#[test]
fn test_LD_A_NNm() {
    let machine = test_cpu(&[0xFA, 0x4B, 0xDE], |cpu| {
        cpu.mem.write_byte(0xDE4B, 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 16);
    assert_eq!(machine.cpu.regs.a, 0x9A);
}

// LDI_HLm_A / LDD_HLm_A : load A into (HL) and increment/decrement HL
#[test]
fn test_LDI_HLm_A() {
    let mut machine = test_cpu(&[0x22], |cpu| {
        cpu.regs.a = 0x9A;
        cpu.regs.set_hl(0x97D3);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0x97D3), 0x9A);
    assert_eq!(machine.cpu.regs.hl(), 0x97D4);
}
#[test]
fn test_LDD_HLm_A() {
    let mut machine = test_cpu(&[0x32], |cpu| {
        cpu.regs.a = 0x9A;
        cpu.regs.set_hl(0x97D3);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0x97D3), 0x9A);
    assert_eq!(machine.cpu.regs.hl(), 0x97D2);
}

// LDI_A_HLm / LDD_A_HLm : load (HL) into A and increment/decrement HL
#[test]
fn test_LDI_A_HLm() {
    let machine = test_cpu(&[0x2A], |cpu| {
        cpu.regs.set_hl(0x97D3);
        cpu.mem.write_byte(cpu.regs.hl(), 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.regs.a, 0x9A);
    assert_eq!(machine.cpu.regs.hl(), 0x97D4);
}
#[test]
fn test_LDD_A_HLm() {
    let machine = test_cpu(&[0x3A], |cpu| {
        cpu.regs.set_hl(0x97D3);
        cpu.mem.write_byte(cpu.regs.hl(), 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.regs.a, 0x9A);
    assert_eq!(machine.cpu.regs.hl(), 0x97D2);
}

// LDH_n_A : load A into (0xFF00 + offset = 8-bit immediate)
#[test]
fn test_LDH_n_A() {
    let mut machine = test_cpu(&[0xE0, 0xC3], |cpu| cpu.regs.a = 0x9A);
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.mem.read_byte(0xFFC3), 0x9A);
}
// LDH_A_n : load (0xFF00 + offset = 8-bit immediate) into A
#[test]
fn test_LDH_A_n() {
    let machine = test_cpu(&[0xF0, 0xC3], |cpu| {
        cpu.mem.write_byte(0xFFC3, 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 12);
    assert_eq!(machine.cpu.regs.a, 0x9A);
}

// LDH_C_A : load A into (0xFF00 + offset = C)
#[test]
fn test_LDH_C_A() {
    let mut machine = test_cpu(&[0xE2], |cpu| {
        cpu.regs.a = 0x9A;
        cpu.regs.c = 0xC3;
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.mem.read_byte(0xFFC3), 0x9A);
}
// LDH_A_C : load (0xFF00 + offset = C) into A
#[test]
fn test_LDH_A_A() {
    let machine = test_cpu(&[0xF2], |cpu| {
        cpu.regs.c = 0xC3;
        cpu.mem.write_byte(0xFFC3, 0x9A);
    });
    assert_eq!(machine.clock_cycles(), 8);
    assert_eq!(machine.cpu.regs.a, 0x9A);
}

// LDHL_SP_n : add signed 8-bit immediate to SP and save the result in HL
#[test]
fn test_LDHL_SP_n() {
    // let machine = test_cpu(&[0xF8, 0xC3], |cpu| {});
}
