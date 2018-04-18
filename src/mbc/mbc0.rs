/// Used by small games who can fit in the 32 KB of ROM and the GameBoy's 8 KB
/// of external RAM.

use super::{CartridgeHeader, MBC};

pub const ROM_SIZE: usize = 0x10000;
pub const ERAM_SIZE: usize = 0x2000;

pub struct MBC0 {
    rom: [u8; ROM_SIZE],
    eram: Option<[u8; ERAM_SIZE]>,
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

            let ram = match CartridgeHeader::ram_size(&data) {
                0 => None,
                ERAM_SIZE => Some([0x00; ERAM_SIZE]),
                n => {
                    error!("MBC0 : invalid external RAM size of {} bytes", n);
                    return Err("MBC0 supports either 0 KB or 8 KB of external RAM");
                }
            };
            Ok(MBC0 { rom, eram: ram })
        }
    }
}

impl MBC for MBC0 {
    fn rom_read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn ram_read(&self, address: u16) -> u8 {
        if self.eram.is_some() {
            self.eram.unwrap()[(address as usize) & 0x1FFF]
        } else {
            0x00
        }
    }

    fn rom_control(&mut self, address: u16, value: u8) {}

    fn ram_write(&mut self, address: u16, value: u8) {
        if self.eram.is_some() {
            self.eram.unwrap()[(address as usize) & 0x1FFF] = value;
        }
    }
}
