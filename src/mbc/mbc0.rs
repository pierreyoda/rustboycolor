/// Used by small games who can fit in the 32 KB of ROM and the GameBoy's 8 KB
/// of external RAM.

use super::MBC;

pub const ROM_SIZE: usize = 65536;

pub struct MBC0 {
    rom: [u8; ROM_SIZE],
}

impl MBC0 {
    pub fn new(data: Vec<u8>) -> ::ResultStr<MBC0> {
        if data.len() > ROM_SIZE {
            Err("ROM too big without any MBC")
        } else {
            let mut rom = [0x00; ROM_SIZE];
            for i in 0..data.len() {
                rom[i] = data[i];
            }
            Ok(MBC0 { rom: rom })
        }
    }
}

impl MBC for MBC0 {
    fn rom_read(&self, address: u16) -> u8 { self.rom[address as usize] }
    fn ram_read(&self, address: u16) -> u8 { 0x0 }
    fn rom_control(&mut self, address: u16, value: u8) { }
    fn ram_write(&mut self, address: u16, value: u8) { }
}
