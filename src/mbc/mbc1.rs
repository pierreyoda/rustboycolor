/// Can address up to 125 ROM banks of 16KB each (i.e. 2MB of ROM at most) and
/// supports 0, 2, 8 or 32 KB of RAM (eventually battery-buffered).
use std::iter;

use super::CartridgeHeader::*;
use super::{CartridgeHeader, MBC};

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    /// The current ROM bank to use when reading in the 0x4000...0x7FFF range.
    /// Possible values are in 0x00...0x7F with the exception of 0x20, 0x40 and
    /// 0x60 which explains the limit of 125 banks (including ROM bank 0x00).
    /// In RAM
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    ram_mode: bool,
}

impl MBC1 {
    pub fn new(data: Vec<u8>) -> crate::ResultStr<MBC1> {
        if data.len() > 0x4000 * 0x7D {
            return Err("MBC1 does not support more than 2MB of ROM");
        }

        let ram_size: usize = match data[CartridgeHeader::address(MBC_Type).unwrap()] {
            // RAM
            0x02 => CartridgeHeader::ram_size(&data),
            // RAM+BATTERY
            0x03 => {
                // TODO: state loading
                CartridgeHeader::ram_size(&data)
            }
            _ => 0,
        };

        Ok(MBC1 {
            rom: data,
            ram: iter::repeat_n(0x00, ram_size).collect(),
            rom_bank: 0x01,
            ram_bank: 0x00,
            ram_enabled: false,
            ram_mode: false,
        })
    }
}

impl MBC for MBC1 {
    fn rom_read(&self, address: u16) -> u8 {
        // ROM bank 00
        if address < 0x4000 {
            self.rom[address as usize]
        }
        // ROM bank 01-7F
        else {
            self.rom[self.rom_bank * 0x4000 + ((address as usize) & 0x3FFF)]
        }
    }

    fn ram_read(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0x00;
        }
        let ram_bank = if self.ram_mode { self.ram_bank } else { 0x00 };
        self.ram[ram_bank * 0x2000 + ((address as usize) & 0x1FFF)]
    }

    fn rom_control(&mut self, address: u16, value: u8) {
        match address {
            // external RAM switch
            0x0000..=0x1FFF => {
                self.ram_enabled = value == 0x0A;
            }
            // ROM bank number : lower 5 bits
            0x2000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x60)
                    + match (address as usize) & 0x1F {
                        0x0 => 0x1,
                        n => n,
                    };
            }
            0x4000..=0x5FFF => {
                let n = (address as usize) & 0x03;
                if self.ram_mode {
                    // RAM bank number
                    self.ram_bank = n;
                } else {
                    // bits 6 and 7 of ROM bank number in ROM mode
                    self.rom_bank = (self.rom_bank & 0x1F) | (n << 5);
                }
            }
            // ROM/RAM mode select
            0x6000..=0x7FFF => {
                self.ram_mode = value == 0x01;
            }
            _ => panic!("MBC1 : cannot write to ROM at {address:0>4X}"),
        }
    }

    fn ram_write(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        let ram_bank = if self.ram_mode { self.ram_bank } else { 0x00 };
        self.ram[ram_bank * 0x2000 + ((address as usize) & 0x1FFF)] = value;
    }
}
